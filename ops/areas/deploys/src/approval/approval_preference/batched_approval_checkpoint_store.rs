use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched approval checkpoints by code.
pub trait BatchedApprovalCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedApprovalCheckpoint>;
    fn save(&self, item: &BatchedApprovalCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedApprovalCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedApprovalCheckpointStore {
    pub items: RefCell<BTreeMap<String, BatchedApprovalCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedApprovalCheckpointStore for MemoryBatchedApprovalCheckpointStore {
    fn find(&self, code: &str) -> Option<BatchedApprovalCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedApprovalCheckpoint) {
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

    fn list(&self) -> Vec<BatchedApprovalCheckpoint> {
        let mut out: Vec<BatchedApprovalCheckpoint> = Vec::new();
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

pub fn new_memory_batched_approval_checkpoint_store() -> Rc<dyn BatchedApprovalCheckpointStore> {
    Rc::new(MemoryBatchedApprovalCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedApprovalCheckpointStore {
    pub inner: Rc<dyn BatchedApprovalCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<BatchedApprovalCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedApprovalCheckpointStore for CachedBatchedApprovalCheckpointStore {
    fn find(&self, code: &str) -> Option<BatchedApprovalCheckpoint> {
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

    fn save(&self, item: &BatchedApprovalCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedApprovalCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_batched_approval_checkpoint_store(
    inner: Rc<dyn BatchedApprovalCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedApprovalCheckpointStore> {
    Rc::new(CachedBatchedApprovalCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedApprovalCheckpoint>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched approval checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalCheckpointQuery {
    pub country: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedApprovalCheckpointQuery {
    pub fn matches(&self, item: &BatchedApprovalCheckpoint) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedApprovalCheckpoint]) -> Vec<BatchedApprovalCheckpoint> {
        let matched = collections::filter(items, |candidate: &BatchedApprovalCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_approval_checkpoints(
    store: Rc<dyn BatchedApprovalCheckpointStore>,
    query: &BatchedApprovalCheckpointQuery,
) -> Vec<BatchedApprovalCheckpoint> {
    query.apply(&store.list())
}
