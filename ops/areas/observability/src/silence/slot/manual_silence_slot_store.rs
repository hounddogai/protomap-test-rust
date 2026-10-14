use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual silence slots by code.
pub trait ManualSilenceSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualSilenceSlot>;
    fn save(&self, item: &ManualSilenceSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualSilenceSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualSilenceSlotStore {
    pub items: RefCell<BTreeMap<String, ManualSilenceSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualSilenceSlotStore for MemoryManualSilenceSlotStore {
    fn find(&self, code: &str) -> Option<ManualSilenceSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualSilenceSlot) {
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

    fn list(&self) -> Vec<ManualSilenceSlot> {
        let mut out: Vec<ManualSilenceSlot> = Vec::new();
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

pub fn new_memory_manual_silence_slot_store() -> Rc<dyn ManualSilenceSlotStore> {
    Rc::new(MemoryManualSilenceSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualSilenceSlotStore {
    pub inner: Rc<dyn ManualSilenceSlotStore>,
    pub cache: RefCell<cache::TtlCache<ManualSilenceSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualSilenceSlotStore for CachedManualSilenceSlotStore {
    fn find(&self, code: &str) -> Option<ManualSilenceSlot> {
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

    fn save(&self, item: &ManualSilenceSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualSilenceSlot> {
        self.inner.list()
    }
}

pub fn new_cached_manual_silence_slot_store(
    inner: Rc<dyn ManualSilenceSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualSilenceSlotStore> {
    Rc::new(CachedManualSilenceSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualSilenceSlot>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual silence slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceSlotQuery {
    pub cluster: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualSilenceSlotQuery {
    pub fn matches(&self, item: &ManualSilenceSlot) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualSilenceSlot]) -> Vec<ManualSilenceSlot> {
        let matched = collections::filter(items, |candidate: &ManualSilenceSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_silence_slots(
    store: Rc<dyn ManualSilenceSlotStore>,
    query: &ManualSilenceSlotQuery,
) -> Vec<ManualSilenceSlot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualSilenceSlotStore> = new_memory_manual_silence_slot_store();
        let item = new_manual_silence_slot("code-167", "name-45");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
