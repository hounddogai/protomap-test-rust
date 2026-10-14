use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending isolation estimates by code.
pub trait PendingIsolationEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingIsolationEstimate>;
    fn save(&self, item: &PendingIsolationEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingIsolationEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingIsolationEstimateStore {
    pub items: RefCell<BTreeMap<String, PendingIsolationEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingIsolationEstimateStore for MemoryPendingIsolationEstimateStore {
    fn find(&self, code: &str) -> Option<PendingIsolationEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingIsolationEstimate) {
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

    fn list(&self) -> Vec<PendingIsolationEstimate> {
        let mut out: Vec<PendingIsolationEstimate> = Vec::new();
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

pub fn new_memory_pending_isolation_estimate_store() -> Rc<dyn PendingIsolationEstimateStore> {
    Rc::new(MemoryPendingIsolationEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingIsolationEstimateStore {
    pub inner: Rc<dyn PendingIsolationEstimateStore>,
    pub cache: RefCell<cache::TtlCache<PendingIsolationEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingIsolationEstimateStore for CachedPendingIsolationEstimateStore {
    fn find(&self, code: &str) -> Option<PendingIsolationEstimate> {
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

    fn save(&self, item: &PendingIsolationEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingIsolationEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_pending_isolation_estimate_store(
    inner: Rc<dyn PendingIsolationEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingIsolationEstimateStore> {
    Rc::new(CachedPendingIsolationEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingIsolationEstimate>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending isolation estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationEstimateQuery {
    pub name: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingIsolationEstimateQuery {
    pub fn matches(&self, item: &PendingIsolationEstimate) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingIsolationEstimate]) -> Vec<PendingIsolationEstimate> {
        let matched = collections::filter(items, |candidate: &PendingIsolationEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_isolation_estimates(
    store: Rc<dyn PendingIsolationEstimateStore>,
    query: &PendingIsolationEstimateQuery,
) -> Vec<PendingIsolationEstimate> {
    query.apply(&store.list())
}
