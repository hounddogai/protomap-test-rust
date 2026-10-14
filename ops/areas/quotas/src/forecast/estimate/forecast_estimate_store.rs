use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast estimates by code.
pub trait ForecastEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastEstimate>;
    fn save(&self, item: &ForecastEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastEstimateStore {
    pub items: RefCell<BTreeMap<String, ForecastEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastEstimateStore for MemoryForecastEstimateStore {
    fn find(&self, code: &str) -> Option<ForecastEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastEstimate) {
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

    fn list(&self) -> Vec<ForecastEstimate> {
        let mut out: Vec<ForecastEstimate> = Vec::new();
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

pub fn new_memory_forecast_estimate_store() -> Rc<dyn ForecastEstimateStore> {
    Rc::new(MemoryForecastEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastEstimateStore {
    pub inner: Rc<dyn ForecastEstimateStore>,
    pub cache: RefCell<cache::TtlCache<ForecastEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastEstimateStore for CachedForecastEstimateStore {
    fn find(&self, code: &str) -> Option<ForecastEstimate> {
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

    fn save(&self, item: &ForecastEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_estimate_store(
    inner: Rc<dyn ForecastEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastEstimateStore> {
    Rc::new(CachedForecastEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastEstimate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEstimateQuery {
    pub region: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastEstimateQuery {
    pub fn matches(&self, item: &ForecastEstimate) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ForecastEstimate]) -> Vec<ForecastEstimate> {
        let matched = collections::filter(items, |candidate: &ForecastEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_estimates(
    store: Rc<dyn ForecastEstimateStore>,
    query: &ForecastEstimateQuery,
) -> Vec<ForecastEstimate> {
    query.apply(&store.list())
}
