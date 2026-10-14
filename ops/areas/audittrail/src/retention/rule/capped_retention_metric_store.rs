use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped retention metrics by code.
pub trait CappedRetentionMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedRetentionMetric>;
    fn save(&self, item: &CappedRetentionMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedRetentionMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedRetentionMetricStore {
    pub items: RefCell<BTreeMap<String, CappedRetentionMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedRetentionMetricStore for MemoryCappedRetentionMetricStore {
    fn find(&self, code: &str) -> Option<CappedRetentionMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedRetentionMetric) {
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

    fn list(&self) -> Vec<CappedRetentionMetric> {
        let mut out: Vec<CappedRetentionMetric> = Vec::new();
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

pub fn new_memory_capped_retention_metric_store() -> Rc<dyn CappedRetentionMetricStore> {
    Rc::new(MemoryCappedRetentionMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedRetentionMetricStore {
    pub inner: Rc<dyn CappedRetentionMetricStore>,
    pub cache: RefCell<cache::TtlCache<CappedRetentionMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedRetentionMetricStore for CachedCappedRetentionMetricStore {
    fn find(&self, code: &str) -> Option<CappedRetentionMetric> {
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

    fn save(&self, item: &CappedRetentionMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedRetentionMetric> {
        self.inner.list()
    }
}

pub fn new_cached_capped_retention_metric_store(
    inner: Rc<dyn CappedRetentionMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedRetentionMetricStore> {
    Rc::new(CachedCappedRetentionMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedRetentionMetric>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped retention metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionMetricQuery {
    pub region: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedRetentionMetricQuery {
    pub fn matches(&self, item: &CappedRetentionMetric) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[CappedRetentionMetric]) -> Vec<CappedRetentionMetric> {
        let matched = collections::filter(items, |candidate: &CappedRetentionMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_retention_metrics(
    store: Rc<dyn CappedRetentionMetricStore>,
    query: &CappedRetentionMetricQuery,
) -> Vec<CappedRetentionMetric> {
    query.apply(&store.list())
}
