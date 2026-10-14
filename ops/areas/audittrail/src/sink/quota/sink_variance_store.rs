use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink variances by code.
pub trait SinkVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkVariance>;
    fn save(&self, item: &SinkVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkVarianceStore {
    pub items: RefCell<BTreeMap<String, SinkVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkVarianceStore for MemorySinkVarianceStore {
    fn find(&self, code: &str) -> Option<SinkVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkVariance) {
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

    fn list(&self) -> Vec<SinkVariance> {
        let mut out: Vec<SinkVariance> = Vec::new();
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

pub fn new_memory_sink_variance_store() -> Rc<dyn SinkVarianceStore> {
    Rc::new(MemorySinkVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkVarianceStore {
    pub inner: Rc<dyn SinkVarianceStore>,
    pub cache: RefCell<cache::TtlCache<SinkVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkVarianceStore for CachedSinkVarianceStore {
    fn find(&self, code: &str) -> Option<SinkVariance> {
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

    fn save(&self, item: &SinkVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkVariance> {
        self.inner.list()
    }
}

pub fn new_cached_sink_variance_store(
    inner: Rc<dyn SinkVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkVarianceStore> {
    Rc::new(CachedSinkVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkVariance>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkVarianceQuery {
    pub cluster: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkVarianceQuery {
    pub fn matches(&self, item: &SinkVariance) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[SinkVariance]) -> Vec<SinkVariance> {
        let matched = collections::filter(items, |candidate: &SinkVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_variances(store: Rc<dyn SinkVarianceStore>, query: &SinkVarianceQuery) -> Vec<SinkVariance> {
    query.apply(&store.list())
}
