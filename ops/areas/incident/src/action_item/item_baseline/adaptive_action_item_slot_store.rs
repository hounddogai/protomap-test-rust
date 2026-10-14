use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive action item slots by code.
pub trait AdaptiveActionItemSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveActionItemSlot>;
    fn save(&self, item: &AdaptiveActionItemSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveActionItemSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveActionItemSlotStore {
    pub items: RefCell<BTreeMap<String, AdaptiveActionItemSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveActionItemSlotStore for MemoryAdaptiveActionItemSlotStore {
    fn find(&self, code: &str) -> Option<AdaptiveActionItemSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveActionItemSlot) {
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

    fn list(&self) -> Vec<AdaptiveActionItemSlot> {
        let mut out: Vec<AdaptiveActionItemSlot> = Vec::new();
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

pub fn new_memory_adaptive_action_item_slot_store() -> Rc<dyn AdaptiveActionItemSlotStore> {
    Rc::new(MemoryAdaptiveActionItemSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveActionItemSlotStore {
    pub inner: Rc<dyn AdaptiveActionItemSlotStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveActionItemSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveActionItemSlotStore for CachedAdaptiveActionItemSlotStore {
    fn find(&self, code: &str) -> Option<AdaptiveActionItemSlot> {
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

    fn save(&self, item: &AdaptiveActionItemSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveActionItemSlot> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_action_item_slot_store(
    inner: Rc<dyn AdaptiveActionItemSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveActionItemSlotStore> {
    Rc::new(CachedAdaptiveActionItemSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveActionItemSlot>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive action item slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveActionItemSlotQuery {
    pub region: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveActionItemSlotQuery {
    pub fn matches(&self, item: &AdaptiveActionItemSlot) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveActionItemSlot]) -> Vec<AdaptiveActionItemSlot> {
        let matched = collections::filter(items, |candidate: &AdaptiveActionItemSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_action_item_slots(
    store: Rc<dyn AdaptiveActionItemSlotStore>,
    query: &AdaptiveActionItemSlotQuery,
) -> Vec<AdaptiveActionItemSlot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveActionItemSlotStore> = new_memory_adaptive_action_item_slot_store();
        let item = new_adaptive_action_item_slot("code-490", "title-436");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
