use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps verifier slots by code.
pub trait VerifierSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<VerifierSlot>;
    fn save(&self, item: &VerifierSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<VerifierSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryVerifierSlotStore {
    pub items: RefCell<BTreeMap<String, VerifierSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl VerifierSlotStore for MemoryVerifierSlotStore {
    fn find(&self, code: &str) -> Option<VerifierSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &VerifierSlot) {
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

    fn list(&self) -> Vec<VerifierSlot> {
        let mut out: Vec<VerifierSlot> = Vec::new();
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

pub fn new_memory_verifier_slot_store() -> Rc<dyn VerifierSlotStore> {
    Rc::new(MemoryVerifierSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedVerifierSlotStore {
    pub inner: Rc<dyn VerifierSlotStore>,
    pub cache: RefCell<cache::TtlCache<VerifierSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl VerifierSlotStore for CachedVerifierSlotStore {
    fn find(&self, code: &str) -> Option<VerifierSlot> {
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

    fn save(&self, item: &VerifierSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<VerifierSlot> {
        self.inner.list()
    }
}

pub fn new_cached_verifier_slot_store(
    inner: Rc<dyn VerifierSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn VerifierSlotStore> {
    Rc::new(CachedVerifierSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<VerifierSlot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects verifier slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSlotQuery {
    pub scope: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl VerifierSlotQuery {
    pub fn matches(&self, item: &VerifierSlot) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[VerifierSlot]) -> Vec<VerifierSlot> {
        let matched = collections::filter(items, |candidate: &VerifierSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_verifier_slots(store: Rc<dyn VerifierSlotStore>, query: &VerifierSlotQuery) -> Vec<VerifierSlot> {
    query.apply(&store.list())
}
