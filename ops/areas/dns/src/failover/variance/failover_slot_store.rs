use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps failover slots by code.
pub trait FailoverSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FailoverSlot>;
    fn save(&self, item: &FailoverSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FailoverSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFailoverSlotStore {
    pub items: RefCell<BTreeMap<String, FailoverSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl FailoverSlotStore for MemoryFailoverSlotStore {
    fn find(&self, code: &str) -> Option<FailoverSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FailoverSlot) {
        if !self.items.borrow().contains_key(&item.code) {
            self.order.borrow_mut().push(item.code.clone());
        }
        self.items.borrow_mut().insert(item.code.clone(), item.clone());
    }

    fn remove(&self, code: &str) -> bool {
        if !self.items.borrow().contains_key(code) {
            return false;
        }
        self.items.borrow_mut().remove(code);
        let mut kept: Vec<String> = Vec::new();
        for existing in self.order.borrow().iter() {
            if existing.as_str() != code {
                kept.push(existing.clone());
            }
        }
        *self.order.borrow_mut() = kept;
        true
    }

    fn list(&self) -> Vec<FailoverSlot> {
        let mut out: Vec<FailoverSlot> = Vec::new();
        for key in self.order.borrow().iter() {
            let found = self.items.borrow().get(key).cloned();
            let Some(found) = found else {
                continue;
            };
            out.push(found);
        }
        out
    }
}

pub fn new_memory_failover_slot_store() -> Rc<dyn FailoverSlotStore> {
    Rc::new(MemoryFailoverSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFailoverSlotStore {
    pub inner: Rc<dyn FailoverSlotStore>,
    pub cache: RefCell<cache::TtlCache<FailoverSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FailoverSlotStore for CachedFailoverSlotStore {
    fn find(&self, code: &str) -> Option<FailoverSlot> {
        let hit = self.cache.borrow_mut().get(code);
        let Some(hit) = hit else {
            self.misses.set(self.misses.get() + 1);
            let Some(found) = self.inner.find(code) else {
                return None;
            };
            self.cache.borrow_mut().put(code, &found);
            return Some(found);
        };
        self.hits.set(self.hits.get() + 1);
        Some(hit)
    }

    fn save(&self, item: &FailoverSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FailoverSlot> {
        self.inner.list()
    }
}

pub fn new_cached_failover_slot_store(
    inner: Rc<dyn FailoverSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FailoverSlotStore> {
    Rc::new(CachedFailoverSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FailoverSlot>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects failover slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSlotQuery {
    pub source: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FailoverSlotQuery {
    pub fn matches(&self, item: &FailoverSlot) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FailoverSlot]) -> Vec<FailoverSlot> {
        let matched = collections::filter(items, |candidate: &FailoverSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_failover_slots(store: Rc<dyn FailoverSlotStore>, query: &FailoverSlotQuery) -> Vec<FailoverSlot> {
    query.apply(&store.list())
}
