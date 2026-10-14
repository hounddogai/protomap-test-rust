use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched health check assignments by code.
pub trait BatchedHealthCheckAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedHealthCheckAssignment>;
    fn save(&self, item: &BatchedHealthCheckAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedHealthCheckAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedHealthCheckAssignmentStore {
    pub items: RefCell<BTreeMap<String, BatchedHealthCheckAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedHealthCheckAssignmentStore for MemoryBatchedHealthCheckAssignmentStore {
    fn find(&self, code: &str) -> Option<BatchedHealthCheckAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedHealthCheckAssignment) {
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

    fn list(&self) -> Vec<BatchedHealthCheckAssignment> {
        let mut out: Vec<BatchedHealthCheckAssignment> = Vec::new();
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

pub fn new_memory_batched_health_check_assignment_store() -> Rc<dyn BatchedHealthCheckAssignmentStore> {
    Rc::new(MemoryBatchedHealthCheckAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedHealthCheckAssignmentStore {
    pub inner: Rc<dyn BatchedHealthCheckAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<BatchedHealthCheckAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedHealthCheckAssignmentStore for CachedBatchedHealthCheckAssignmentStore {
    fn find(&self, code: &str) -> Option<BatchedHealthCheckAssignment> {
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

    fn save(&self, item: &BatchedHealthCheckAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedHealthCheckAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_batched_health_check_assignment_store(
    inner: Rc<dyn BatchedHealthCheckAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedHealthCheckAssignmentStore> {
    Rc::new(CachedBatchedHealthCheckAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedHealthCheckAssignment>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched health check assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckAssignmentQuery {
    pub name: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedHealthCheckAssignmentQuery {
    pub fn matches(&self, item: &BatchedHealthCheckAssignment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[BatchedHealthCheckAssignment]) -> Vec<BatchedHealthCheckAssignment> {
        let matched = collections::filter(items, |candidate: &BatchedHealthCheckAssignment| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_health_check_assignments(
    store: Rc<dyn BatchedHealthCheckAssignmentStore>,
    query: &BatchedHealthCheckAssignmentQuery,
) -> Vec<BatchedHealthCheckAssignment> {
    query.apply(&store.list())
}
