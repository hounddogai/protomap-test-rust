use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global sink slots by code.
pub trait GlobalSinkSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalSinkSlot>;
    fn save(&self, item: &GlobalSinkSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalSinkSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalSinkSlotStore {
    pub items: RefCell<BTreeMap<String, GlobalSinkSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalSinkSlotStore for MemoryGlobalSinkSlotStore {
    fn find(&self, code: &str) -> Option<GlobalSinkSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalSinkSlot) {
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

    fn list(&self) -> Vec<GlobalSinkSlot> {
        let mut out: Vec<GlobalSinkSlot> = Vec::new();
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

pub fn new_memory_global_sink_slot_store() -> Rc<dyn GlobalSinkSlotStore> {
    Rc::new(MemoryGlobalSinkSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalSinkSlotStore {
    pub inner: Rc<dyn GlobalSinkSlotStore>,
    pub cache: RefCell<cache::TtlCache<GlobalSinkSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalSinkSlotStore for CachedGlobalSinkSlotStore {
    fn find(&self, code: &str) -> Option<GlobalSinkSlot> {
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

    fn save(&self, item: &GlobalSinkSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalSinkSlot> {
        self.inner.list()
    }
}

pub fn new_cached_global_sink_slot_store(
    inner: Rc<dyn GlobalSinkSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalSinkSlotStore> {
    Rc::new(CachedGlobalSinkSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalSinkSlot>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global sink slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkSlotQuery {
    pub country: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalSinkSlotQuery {
    pub fn matches(&self, item: &GlobalSinkSlot) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalSinkSlot]) -> Vec<GlobalSinkSlot> {
        let matched = collections::filter(items, |candidate: &GlobalSinkSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_sink_slots(store: Rc<dyn GlobalSinkSlotStore>, query: &GlobalSinkSlotQuery) -> Vec<GlobalSinkSlot> {
    query.apply(&store.list())
}
