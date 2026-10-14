use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention forecasts by code.
pub trait RetentionForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionForecast>;
    fn save(&self, item: &RetentionForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionForecastStore {
    pub items: RefCell<BTreeMap<String, RetentionForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionForecastStore for MemoryRetentionForecastStore {
    fn find(&self, code: &str) -> Option<RetentionForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionForecast) {
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

    fn list(&self) -> Vec<RetentionForecast> {
        let mut out: Vec<RetentionForecast> = Vec::new();
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

pub fn new_memory_retention_forecast_store() -> Rc<dyn RetentionForecastStore> {
    Rc::new(MemoryRetentionForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionForecastStore {
    pub inner: Rc<dyn RetentionForecastStore>,
    pub cache: RefCell<cache::TtlCache<RetentionForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionForecastStore for CachedRetentionForecastStore {
    fn find(&self, code: &str) -> Option<RetentionForecast> {
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

    fn save(&self, item: &RetentionForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionForecast> {
        self.inner.list()
    }
}

pub fn new_cached_retention_forecast_store(
    inner: Rc<dyn RetentionForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionForecastStore> {
    Rc::new(CachedRetentionForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionForecast>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionForecastQuery {
    pub label: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionForecastQuery {
    pub fn matches(&self, item: &RetentionForecast) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RetentionForecast]) -> Vec<RetentionForecast> {
        let matched = collections::filter(items, |candidate: &RetentionForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_forecasts(
    store: Rc<dyn RetentionForecastStore>,
    query: &RetentionForecastQuery,
) -> Vec<RetentionForecast> {
    query.apply(&store.list())
}
