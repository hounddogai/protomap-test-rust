use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback rollout slots by code.
pub trait FallbackRolloutSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackRolloutSlot>;
    fn save(&self, item: &FallbackRolloutSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackRolloutSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackRolloutSlotStore {
    pub items: RefCell<BTreeMap<String, FallbackRolloutSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackRolloutSlotStore for MemoryFallbackRolloutSlotStore {
    fn find(&self, code: &str) -> Option<FallbackRolloutSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackRolloutSlot) {
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

    fn list(&self) -> Vec<FallbackRolloutSlot> {
        let mut out: Vec<FallbackRolloutSlot> = Vec::new();
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

pub fn new_memory_fallback_rollout_slot_store() -> Rc<dyn FallbackRolloutSlotStore> {
    Rc::new(MemoryFallbackRolloutSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackRolloutSlotStore {
    pub inner: Rc<dyn FallbackRolloutSlotStore>,
    pub cache: RefCell<cache::TtlCache<FallbackRolloutSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackRolloutSlotStore for CachedFallbackRolloutSlotStore {
    fn find(&self, code: &str) -> Option<FallbackRolloutSlot> {
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

    fn save(&self, item: &FallbackRolloutSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackRolloutSlot> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_rollout_slot_store(
    inner: Rc<dyn FallbackRolloutSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackRolloutSlotStore> {
    Rc::new(CachedFallbackRolloutSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackRolloutSlot>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback rollout slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutSlotQuery {
    pub channel: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackRolloutSlotQuery {
    pub fn matches(&self, item: &FallbackRolloutSlot) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackRolloutSlot]) -> Vec<FallbackRolloutSlot> {
        let matched = collections::filter(items, |candidate: &FallbackRolloutSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_rollout_slots(
    store: Rc<dyn FallbackRolloutSlotStore>,
    query: &FallbackRolloutSlotQuery,
) -> Vec<FallbackRolloutSlot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackRolloutSlotStore> = new_memory_fallback_rollout_slot_store();
        let item = new_fallback_rollout_slot("code-839", "label-323");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
