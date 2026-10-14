use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query thresholds by code.
pub trait QueryThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryThreshold>;
    fn save(&self, item: &QueryThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryThresholdStore {
    pub items: RefCell<BTreeMap<String, QueryThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryThresholdStore for MemoryQueryThresholdStore {
    fn find(&self, code: &str) -> Option<QueryThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryThreshold) {
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

    fn list(&self) -> Vec<QueryThreshold> {
        let mut out: Vec<QueryThreshold> = Vec::new();
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

pub fn new_memory_query_threshold_store() -> Rc<dyn QueryThresholdStore> {
    Rc::new(MemoryQueryThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryThresholdStore {
    pub inner: Rc<dyn QueryThresholdStore>,
    pub cache: RefCell<cache::TtlCache<QueryThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryThresholdStore for CachedQueryThresholdStore {
    fn find(&self, code: &str) -> Option<QueryThreshold> {
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

    fn save(&self, item: &QueryThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_query_threshold_store(
    inner: Rc<dyn QueryThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryThresholdStore> {
    Rc::new(CachedQueryThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryThreshold>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryThresholdQuery {
    pub country: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryThresholdQuery {
    pub fn matches(&self, item: &QueryThreshold) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QueryThreshold]) -> Vec<QueryThreshold> {
        let matched = collections::filter(items, |candidate: &QueryThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_thresholds(store: Rc<dyn QueryThresholdStore>, query: &QueryThresholdQuery) -> Vec<QueryThreshold> {
    query.apply(&store.list())
}
