use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast metrics by code.
pub trait ForecastMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastMetric>;
    fn save(&self, item: &ForecastMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastMetricStore {
    pub items: RefCell<BTreeMap<String, ForecastMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastMetricStore for MemoryForecastMetricStore {
    fn find(&self, code: &str) -> Option<ForecastMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastMetric) {
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

    fn list(&self) -> Vec<ForecastMetric> {
        let mut out: Vec<ForecastMetric> = Vec::new();
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

pub fn new_memory_forecast_metric_store() -> Rc<dyn ForecastMetricStore> {
    Rc::new(MemoryForecastMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastMetricStore {
    pub inner: Rc<dyn ForecastMetricStore>,
    pub cache: RefCell<cache::TtlCache<ForecastMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastMetricStore for CachedForecastMetricStore {
    fn find(&self, code: &str) -> Option<ForecastMetric> {
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

    fn save(&self, item: &ForecastMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastMetric> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_metric_store(
    inner: Rc<dyn ForecastMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastMetricStore> {
    Rc::new(CachedForecastMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastMetric>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMetricQuery {
    pub name: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastMetricQuery {
    pub fn matches(&self, item: &ForecastMetric) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ForecastMetric]) -> Vec<ForecastMetric> {
        let matched = collections::filter(items, |candidate: &ForecastMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_metrics(store: Rc<dyn ForecastMetricStore>, query: &ForecastMetricQuery) -> Vec<ForecastMetric> {
    query.apply(&store.list())
}
