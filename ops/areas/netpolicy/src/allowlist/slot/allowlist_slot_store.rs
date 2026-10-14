use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps allowlist slots by code.
pub trait AllowlistSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AllowlistSlot>;
    fn save(&self, item: &AllowlistSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AllowlistSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAllowlistSlotStore {
    pub items: RefCell<BTreeMap<String, AllowlistSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl AllowlistSlotStore for MemoryAllowlistSlotStore {
    fn find(&self, code: &str) -> Option<AllowlistSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AllowlistSlot) {
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

    fn list(&self) -> Vec<AllowlistSlot> {
        let mut out: Vec<AllowlistSlot> = Vec::new();
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

pub fn new_memory_allowlist_slot_store() -> Rc<dyn AllowlistSlotStore> {
    Rc::new(MemoryAllowlistSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAllowlistSlotStore {
    pub inner: Rc<dyn AllowlistSlotStore>,
    pub cache: RefCell<cache::TtlCache<AllowlistSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AllowlistSlotStore for CachedAllowlistSlotStore {
    fn find(&self, code: &str) -> Option<AllowlistSlot> {
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

    fn save(&self, item: &AllowlistSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AllowlistSlot> {
        self.inner.list()
    }
}

pub fn new_cached_allowlist_slot_store(
    inner: Rc<dyn AllowlistSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AllowlistSlotStore> {
    Rc::new(CachedAllowlistSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AllowlistSlot>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects allowlist slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSlotQuery {
    pub name: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AllowlistSlotQuery {
    pub fn matches(&self, item: &AllowlistSlot) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AllowlistSlot]) -> Vec<AllowlistSlot> {
        let matched = collections::filter(items, |candidate: &AllowlistSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_allowlist_slots(store: Rc<dyn AllowlistSlotStore>, query: &AllowlistSlotQuery) -> Vec<AllowlistSlot> {
    query.apply(&store.list())
}
