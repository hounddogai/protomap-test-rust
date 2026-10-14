use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback retention forecasts by code.
pub trait FallbackRetentionForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackRetentionForecast>;
    fn save(&self, item: &FallbackRetentionForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackRetentionForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackRetentionForecastStore {
    pub items: RefCell<BTreeMap<String, FallbackRetentionForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackRetentionForecastStore for MemoryFallbackRetentionForecastStore {
    fn find(&self, code: &str) -> Option<FallbackRetentionForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackRetentionForecast) {
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

    fn list(&self) -> Vec<FallbackRetentionForecast> {
        let mut out: Vec<FallbackRetentionForecast> = Vec::new();
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

pub fn new_memory_fallback_retention_forecast_store() -> Rc<dyn FallbackRetentionForecastStore> {
    Rc::new(MemoryFallbackRetentionForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackRetentionForecastStore {
    pub inner: Rc<dyn FallbackRetentionForecastStore>,
    pub cache: RefCell<cache::TtlCache<FallbackRetentionForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackRetentionForecastStore for CachedFallbackRetentionForecastStore {
    fn find(&self, code: &str) -> Option<FallbackRetentionForecast> {
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

    fn save(&self, item: &FallbackRetentionForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackRetentionForecast> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_retention_forecast_store(
    inner: Rc<dyn FallbackRetentionForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackRetentionForecastStore> {
    Rc::new(CachedFallbackRetentionForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackRetentionForecast>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback retention forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionForecastQuery {
    pub scope: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackRetentionForecastQuery {
    pub fn matches(&self, item: &FallbackRetentionForecast) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackRetentionForecast]) -> Vec<FallbackRetentionForecast> {
        let matched = collections::filter(items, |candidate: &FallbackRetentionForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_retention_forecasts(
    store: Rc<dyn FallbackRetentionForecastStore>,
    query: &FallbackRetentionForecastQuery,
) -> Vec<FallbackRetentionForecast> {
    query.apply(&store.list())
}
