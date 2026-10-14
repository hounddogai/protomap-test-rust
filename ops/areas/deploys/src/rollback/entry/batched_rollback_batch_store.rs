use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched rollback batches by code.
pub trait BatchedRollbackBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedRollbackBatch>;
    fn save(&self, item: &BatchedRollbackBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedRollbackBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedRollbackBatchStore {
    pub items: RefCell<BTreeMap<String, BatchedRollbackBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedRollbackBatchStore for MemoryBatchedRollbackBatchStore {
    fn find(&self, code: &str) -> Option<BatchedRollbackBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedRollbackBatch) {
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

    fn list(&self) -> Vec<BatchedRollbackBatch> {
        let mut out: Vec<BatchedRollbackBatch> = Vec::new();
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

pub fn new_memory_batched_rollback_batch_store() -> Rc<dyn BatchedRollbackBatchStore> {
    Rc::new(MemoryBatchedRollbackBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedRollbackBatchStore {
    pub inner: Rc<dyn BatchedRollbackBatchStore>,
    pub cache: RefCell<cache::TtlCache<BatchedRollbackBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedRollbackBatchStore for CachedBatchedRollbackBatchStore {
    fn find(&self, code: &str) -> Option<BatchedRollbackBatch> {
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

    fn save(&self, item: &BatchedRollbackBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedRollbackBatch> {
        self.inner.list()
    }
}

pub fn new_cached_batched_rollback_batch_store(
    inner: Rc<dyn BatchedRollbackBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedRollbackBatchStore> {
    Rc::new(CachedBatchedRollbackBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedRollbackBatch>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched rollback batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatchQuery {
    pub unit: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedRollbackBatchQuery {
    pub fn matches(&self, item: &BatchedRollbackBatch) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedRollbackBatch]) -> Vec<BatchedRollbackBatch> {
        let matched = collections::filter(items, |candidate: &BatchedRollbackBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_rollback_batches(
    store: Rc<dyn BatchedRollbackBatchStore>,
    query: &BatchedRollbackBatchQuery,
) -> Vec<BatchedRollbackBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedRollbackBatchStore> = new_memory_batched_rollback_batch_store();
        let item = new_batched_rollback_batch("code-500", "label-79");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
