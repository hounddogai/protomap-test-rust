use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched egress requests by code.
pub trait BatchedEgressRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedEgressRequest>;
    fn save(&self, item: &BatchedEgressRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedEgressRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedEgressRequestStore {
    pub items: RefCell<BTreeMap<String, BatchedEgressRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedEgressRequestStore for MemoryBatchedEgressRequestStore {
    fn find(&self, code: &str) -> Option<BatchedEgressRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedEgressRequest) {
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

    fn list(&self) -> Vec<BatchedEgressRequest> {
        let mut out: Vec<BatchedEgressRequest> = Vec::new();
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

pub fn new_memory_batched_egress_request_store() -> Rc<dyn BatchedEgressRequestStore> {
    Rc::new(MemoryBatchedEgressRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedEgressRequestStore {
    pub inner: Rc<dyn BatchedEgressRequestStore>,
    pub cache: RefCell<cache::TtlCache<BatchedEgressRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedEgressRequestStore for CachedBatchedEgressRequestStore {
    fn find(&self, code: &str) -> Option<BatchedEgressRequest> {
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

    fn save(&self, item: &BatchedEgressRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedEgressRequest> {
        self.inner.list()
    }
}

pub fn new_cached_batched_egress_request_store(
    inner: Rc<dyn BatchedEgressRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedEgressRequestStore> {
    Rc::new(CachedBatchedEgressRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedEgressRequest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched egress requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEgressRequestQuery {
    pub title: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedEgressRequestQuery {
    pub fn matches(&self, item: &BatchedEgressRequest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedEgressRequest]) -> Vec<BatchedEgressRequest> {
        let matched = collections::filter(items, |candidate: &BatchedEgressRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_egress_requests(
    store: Rc<dyn BatchedEgressRequestStore>,
    query: &BatchedEgressRequestQuery,
) -> Vec<BatchedEgressRequest> {
    query.apply(&store.list())
}
