use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched allowlist assignments by code.
pub trait BatchedAllowlistAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedAllowlistAssignment>;
    fn save(&self, item: &BatchedAllowlistAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedAllowlistAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedAllowlistAssignmentStore {
    pub items: RefCell<BTreeMap<String, BatchedAllowlistAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedAllowlistAssignmentStore for MemoryBatchedAllowlistAssignmentStore {
    fn find(&self, code: &str) -> Option<BatchedAllowlistAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedAllowlistAssignment) {
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

    fn list(&self) -> Vec<BatchedAllowlistAssignment> {
        let mut out: Vec<BatchedAllowlistAssignment> = Vec::new();
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

pub fn new_memory_batched_allowlist_assignment_store() -> Rc<dyn BatchedAllowlistAssignmentStore> {
    Rc::new(MemoryBatchedAllowlistAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedAllowlistAssignmentStore {
    pub inner: Rc<dyn BatchedAllowlistAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<BatchedAllowlistAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedAllowlistAssignmentStore for CachedBatchedAllowlistAssignmentStore {
    fn find(&self, code: &str) -> Option<BatchedAllowlistAssignment> {
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

    fn save(&self, item: &BatchedAllowlistAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedAllowlistAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_batched_allowlist_assignment_store(
    inner: Rc<dyn BatchedAllowlistAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedAllowlistAssignmentStore> {
    Rc::new(CachedBatchedAllowlistAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedAllowlistAssignment>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched allowlist assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistAssignmentQuery {
    pub channel: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedAllowlistAssignmentQuery {
    pub fn matches(&self, item: &BatchedAllowlistAssignment) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedAllowlistAssignment]) -> Vec<BatchedAllowlistAssignment> {
        let matched = collections::filter(items, |candidate: &BatchedAllowlistAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_allowlist_assignments(
    store: Rc<dyn BatchedAllowlistAssignmentStore>,
    query: &BatchedAllowlistAssignmentQuery,
) -> Vec<BatchedAllowlistAssignment> {
    query.apply(&store.list())
}
