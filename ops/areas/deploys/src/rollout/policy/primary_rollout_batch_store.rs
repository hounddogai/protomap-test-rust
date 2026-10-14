use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary rollout batches by code.
pub trait PrimaryRolloutBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryRolloutBatch>;
    fn save(&self, item: &PrimaryRolloutBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryRolloutBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryRolloutBatchStore {
    pub items: RefCell<BTreeMap<String, PrimaryRolloutBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryRolloutBatchStore for MemoryPrimaryRolloutBatchStore {
    fn find(&self, code: &str) -> Option<PrimaryRolloutBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryRolloutBatch) {
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

    fn list(&self) -> Vec<PrimaryRolloutBatch> {
        let mut out: Vec<PrimaryRolloutBatch> = Vec::new();
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

pub fn new_memory_primary_rollout_batch_store() -> Rc<dyn PrimaryRolloutBatchStore> {
    Rc::new(MemoryPrimaryRolloutBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryRolloutBatchStore {
    pub inner: Rc<dyn PrimaryRolloutBatchStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryRolloutBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryRolloutBatchStore for CachedPrimaryRolloutBatchStore {
    fn find(&self, code: &str) -> Option<PrimaryRolloutBatch> {
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

    fn save(&self, item: &PrimaryRolloutBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryRolloutBatch> {
        self.inner.list()
    }
}

pub fn new_cached_primary_rollout_batch_store(
    inner: Rc<dyn PrimaryRolloutBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryRolloutBatchStore> {
    Rc::new(CachedPrimaryRolloutBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryRolloutBatch>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary rollout batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutBatchQuery {
    pub unit: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryRolloutBatchQuery {
    pub fn matches(&self, item: &PrimaryRolloutBatch) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryRolloutBatch]) -> Vec<PrimaryRolloutBatch> {
        let matched = collections::filter(items, |candidate: &PrimaryRolloutBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_rollout_batches(
    store: Rc<dyn PrimaryRolloutBatchStore>,
    query: &PrimaryRolloutBatchQuery,
) -> Vec<PrimaryRolloutBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryRolloutBatchStore> = new_memory_primary_rollout_batch_store();
        let item = new_primary_rollout_batch("code-754", "name-800");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
