use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict release slots by code.
pub trait StrictReleaseSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictReleaseSlot>;
    fn save(&self, item: &StrictReleaseSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictReleaseSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictReleaseSlotStore {
    pub items: RefCell<BTreeMap<String, StrictReleaseSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictReleaseSlotStore for MemoryStrictReleaseSlotStore {
    fn find(&self, code: &str) -> Option<StrictReleaseSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictReleaseSlot) {
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

    fn list(&self) -> Vec<StrictReleaseSlot> {
        let mut out: Vec<StrictReleaseSlot> = Vec::new();
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

pub fn new_memory_strict_release_slot_store() -> Rc<dyn StrictReleaseSlotStore> {
    Rc::new(MemoryStrictReleaseSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictReleaseSlotStore {
    pub inner: Rc<dyn StrictReleaseSlotStore>,
    pub cache: RefCell<cache::TtlCache<StrictReleaseSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictReleaseSlotStore for CachedStrictReleaseSlotStore {
    fn find(&self, code: &str) -> Option<StrictReleaseSlot> {
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

    fn save(&self, item: &StrictReleaseSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictReleaseSlot> {
        self.inner.list()
    }
}

pub fn new_cached_strict_release_slot_store(
    inner: Rc<dyn StrictReleaseSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictReleaseSlotStore> {
    Rc::new(CachedStrictReleaseSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictReleaseSlot>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict release slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseSlotQuery {
    pub unit: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictReleaseSlotQuery {
    pub fn matches(&self, item: &StrictReleaseSlot) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictReleaseSlot]) -> Vec<StrictReleaseSlot> {
        let matched = collections::filter(items, |candidate: &StrictReleaseSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_release_slots(
    store: Rc<dyn StrictReleaseSlotStore>,
    query: &StrictReleaseSlotQuery,
) -> Vec<StrictReleaseSlot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictReleaseSlotStore> = new_memory_strict_release_slot_store();
        let item = new_strict_release_slot("code-458", "title-902");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
