use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps ttl policy metrics by code.
pub trait TtlPolicyMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TtlPolicyMetric>;
    fn save(&self, item: &TtlPolicyMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TtlPolicyMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTtlPolicyMetricStore {
    pub items: RefCell<BTreeMap<String, TtlPolicyMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl TtlPolicyMetricStore for MemoryTtlPolicyMetricStore {
    fn find(&self, code: &str) -> Option<TtlPolicyMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TtlPolicyMetric) {
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

    fn list(&self) -> Vec<TtlPolicyMetric> {
        let mut out: Vec<TtlPolicyMetric> = Vec::new();
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

pub fn new_memory_ttl_policy_metric_store() -> Rc<dyn TtlPolicyMetricStore> {
    Rc::new(MemoryTtlPolicyMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTtlPolicyMetricStore {
    pub inner: Rc<dyn TtlPolicyMetricStore>,
    pub cache: RefCell<cache::TtlCache<TtlPolicyMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TtlPolicyMetricStore for CachedTtlPolicyMetricStore {
    fn find(&self, code: &str) -> Option<TtlPolicyMetric> {
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

    fn save(&self, item: &TtlPolicyMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TtlPolicyMetric> {
        self.inner.list()
    }
}

pub fn new_cached_ttl_policy_metric_store(
    inner: Rc<dyn TtlPolicyMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TtlPolicyMetricStore> {
    Rc::new(CachedTtlPolicyMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TtlPolicyMetric>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects ttl policy metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMetricQuery {
    pub source: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TtlPolicyMetricQuery {
    pub fn matches(&self, item: &TtlPolicyMetric) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TtlPolicyMetric]) -> Vec<TtlPolicyMetric> {
        let matched = collections::filter(items, |candidate: &TtlPolicyMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_ttl_policy_metrics(
    store: Rc<dyn TtlPolicyMetricStore>,
    query: &TtlPolicyMetricQuery,
) -> Vec<TtlPolicyMetric> {
    query.apply(&store.list())
}
