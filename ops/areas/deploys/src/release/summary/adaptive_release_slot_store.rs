use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive release slots by code.
pub trait AdaptiveReleaseSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveReleaseSlot>;
    fn save(&self, item: &AdaptiveReleaseSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveReleaseSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveReleaseSlotStore {
    pub items: RefCell<BTreeMap<String, AdaptiveReleaseSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveReleaseSlotStore for MemoryAdaptiveReleaseSlotStore {
    fn find(&self, code: &str) -> Option<AdaptiveReleaseSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveReleaseSlot) {
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

    fn list(&self) -> Vec<AdaptiveReleaseSlot> {
        let mut out: Vec<AdaptiveReleaseSlot> = Vec::new();
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

pub fn new_memory_adaptive_release_slot_store() -> Rc<dyn AdaptiveReleaseSlotStore> {
    Rc::new(MemoryAdaptiveReleaseSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveReleaseSlotStore {
    pub inner: Rc<dyn AdaptiveReleaseSlotStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveReleaseSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveReleaseSlotStore for CachedAdaptiveReleaseSlotStore {
    fn find(&self, code: &str) -> Option<AdaptiveReleaseSlot> {
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

    fn save(&self, item: &AdaptiveReleaseSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveReleaseSlot> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_release_slot_store(
    inner: Rc<dyn AdaptiveReleaseSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveReleaseSlotStore> {
    Rc::new(CachedAdaptiveReleaseSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveReleaseSlot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive release slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseSlotQuery {
    pub channel: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveReleaseSlotQuery {
    pub fn matches(&self, item: &AdaptiveReleaseSlot) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveReleaseSlot]) -> Vec<AdaptiveReleaseSlot> {
        let matched = collections::filter(items, |candidate: &AdaptiveReleaseSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_release_slots(
    store: Rc<dyn AdaptiveReleaseSlotStore>,
    query: &AdaptiveReleaseSlotQuery,
) -> Vec<AdaptiveReleaseSlot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveReleaseSlotStore> = new_memory_adaptive_release_slot_store();
        let item = new_adaptive_release_slot("code-166", "label-15");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
