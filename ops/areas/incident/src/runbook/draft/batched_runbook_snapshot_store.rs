use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched runbook snapshots by code.
pub trait BatchedRunbookSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedRunbookSnapshot>;
    fn save(&self, item: &BatchedRunbookSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedRunbookSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedRunbookSnapshotStore {
    pub items: RefCell<BTreeMap<String, BatchedRunbookSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedRunbookSnapshotStore for MemoryBatchedRunbookSnapshotStore {
    fn find(&self, code: &str) -> Option<BatchedRunbookSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedRunbookSnapshot) {
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

    fn list(&self) -> Vec<BatchedRunbookSnapshot> {
        let mut out: Vec<BatchedRunbookSnapshot> = Vec::new();
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

pub fn new_memory_batched_runbook_snapshot_store() -> Rc<dyn BatchedRunbookSnapshotStore> {
    Rc::new(MemoryBatchedRunbookSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedRunbookSnapshotStore {
    pub inner: Rc<dyn BatchedRunbookSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<BatchedRunbookSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedRunbookSnapshotStore for CachedBatchedRunbookSnapshotStore {
    fn find(&self, code: &str) -> Option<BatchedRunbookSnapshot> {
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

    fn save(&self, item: &BatchedRunbookSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedRunbookSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_batched_runbook_snapshot_store(
    inner: Rc<dyn BatchedRunbookSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedRunbookSnapshotStore> {
    Rc::new(CachedBatchedRunbookSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedRunbookSnapshot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched runbook snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSnapshotQuery {
    pub category: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedRunbookSnapshotQuery {
    pub fn matches(&self, item: &BatchedRunbookSnapshot) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedRunbookSnapshot]) -> Vec<BatchedRunbookSnapshot> {
        let matched = collections::filter(items, |candidate: &BatchedRunbookSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_runbook_snapshots(
    store: Rc<dyn BatchedRunbookSnapshotStore>,
    query: &BatchedRunbookSnapshotQuery,
) -> Vec<BatchedRunbookSnapshot> {
    query.apply(&store.list())
}
