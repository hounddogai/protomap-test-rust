use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query variances by code.
pub trait QueryVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryVariance>;
    fn save(&self, item: &QueryVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryVarianceStore {
    pub items: RefCell<BTreeMap<String, QueryVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryVarianceStore for MemoryQueryVarianceStore {
    fn find(&self, code: &str) -> Option<QueryVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryVariance) {
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

    fn list(&self) -> Vec<QueryVariance> {
        let mut out: Vec<QueryVariance> = Vec::new();
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

pub fn new_memory_query_variance_store() -> Rc<dyn QueryVarianceStore> {
    Rc::new(MemoryQueryVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryVarianceStore {
    pub inner: Rc<dyn QueryVarianceStore>,
    pub cache: RefCell<cache::TtlCache<QueryVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryVarianceStore for CachedQueryVarianceStore {
    fn find(&self, code: &str) -> Option<QueryVariance> {
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

    fn save(&self, item: &QueryVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryVariance> {
        self.inner.list()
    }
}

pub fn new_cached_query_variance_store(
    inner: Rc<dyn QueryVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryVarianceStore> {
    Rc::new(CachedQueryVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryVariance>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryVarianceQuery {
    pub label: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryVarianceQuery {
    pub fn matches(&self, item: &QueryVariance) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QueryVariance]) -> Vec<QueryVariance> {
        let matched = collections::filter(items, |candidate: &QueryVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_variances(store: Rc<dyn QueryVarianceStore>, query: &QueryVarianceQuery) -> Vec<QueryVariance> {
    query.apply(&store.list())
}
