use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps failover estimates by code.
pub trait FailoverEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FailoverEstimate>;
    fn save(&self, item: &FailoverEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FailoverEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFailoverEstimateStore {
    pub items: RefCell<BTreeMap<String, FailoverEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl FailoverEstimateStore for MemoryFailoverEstimateStore {
    fn find(&self, code: &str) -> Option<FailoverEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FailoverEstimate) {
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

    fn list(&self) -> Vec<FailoverEstimate> {
        let mut out: Vec<FailoverEstimate> = Vec::new();
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

pub fn new_memory_failover_estimate_store() -> Rc<dyn FailoverEstimateStore> {
    Rc::new(MemoryFailoverEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFailoverEstimateStore {
    pub inner: Rc<dyn FailoverEstimateStore>,
    pub cache: RefCell<cache::TtlCache<FailoverEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FailoverEstimateStore for CachedFailoverEstimateStore {
    fn find(&self, code: &str) -> Option<FailoverEstimate> {
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

    fn save(&self, item: &FailoverEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FailoverEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_failover_estimate_store(
    inner: Rc<dyn FailoverEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FailoverEstimateStore> {
    Rc::new(CachedFailoverEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FailoverEstimate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects failover estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEstimateQuery {
    pub region: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FailoverEstimateQuery {
    pub fn matches(&self, item: &FailoverEstimate) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FailoverEstimate]) -> Vec<FailoverEstimate> {
        let matched = collections::filter(items, |candidate: &FailoverEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_failover_estimates(
    store: Rc<dyn FailoverEstimateStore>,
    query: &FailoverEstimateQuery,
) -> Vec<FailoverEstimate> {
    query.apply(&store.list())
}
