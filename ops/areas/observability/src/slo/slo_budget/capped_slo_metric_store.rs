use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped slo metrics by code.
pub trait CappedSloMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedSloMetric>;
    fn save(&self, item: &CappedSloMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedSloMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedSloMetricStore {
    pub items: RefCell<BTreeMap<String, CappedSloMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedSloMetricStore for MemoryCappedSloMetricStore {
    fn find(&self, code: &str) -> Option<CappedSloMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedSloMetric) {
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

    fn list(&self) -> Vec<CappedSloMetric> {
        let mut out: Vec<CappedSloMetric> = Vec::new();
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

pub fn new_memory_capped_slo_metric_store() -> Rc<dyn CappedSloMetricStore> {
    Rc::new(MemoryCappedSloMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedSloMetricStore {
    pub inner: Rc<dyn CappedSloMetricStore>,
    pub cache: RefCell<cache::TtlCache<CappedSloMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedSloMetricStore for CachedCappedSloMetricStore {
    fn find(&self, code: &str) -> Option<CappedSloMetric> {
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

    fn save(&self, item: &CappedSloMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedSloMetric> {
        self.inner.list()
    }
}

pub fn new_cached_capped_slo_metric_store(
    inner: Rc<dyn CappedSloMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedSloMetricStore> {
    Rc::new(CachedCappedSloMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedSloMetric>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped slo metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloMetricQuery {
    pub title: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedSloMetricQuery {
    pub fn matches(&self, item: &CappedSloMetric) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[CappedSloMetric]) -> Vec<CappedSloMetric> {
        let matched = collections::filter(items, |candidate: &CappedSloMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_slo_metrics(
    store: Rc<dyn CappedSloMetricStore>,
    query: &CappedSloMetricQuery,
) -> Vec<CappedSloMetric> {
    query.apply(&store.list())
}
