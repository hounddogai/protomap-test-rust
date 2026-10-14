use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending slo forecasts by code.
pub trait PendingSloForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingSloForecast>;
    fn save(&self, item: &PendingSloForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingSloForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingSloForecastStore {
    pub items: RefCell<BTreeMap<String, PendingSloForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingSloForecastStore for MemoryPendingSloForecastStore {
    fn find(&self, code: &str) -> Option<PendingSloForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingSloForecast) {
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

    fn list(&self) -> Vec<PendingSloForecast> {
        let mut out: Vec<PendingSloForecast> = Vec::new();
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

pub fn new_memory_pending_slo_forecast_store() -> Rc<dyn PendingSloForecastStore> {
    Rc::new(MemoryPendingSloForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingSloForecastStore {
    pub inner: Rc<dyn PendingSloForecastStore>,
    pub cache: RefCell<cache::TtlCache<PendingSloForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingSloForecastStore for CachedPendingSloForecastStore {
    fn find(&self, code: &str) -> Option<PendingSloForecast> {
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

    fn save(&self, item: &PendingSloForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingSloForecast> {
        self.inner.list()
    }
}

pub fn new_cached_pending_slo_forecast_store(
    inner: Rc<dyn PendingSloForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingSloForecastStore> {
    Rc::new(CachedPendingSloForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingSloForecast>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending slo forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloForecastQuery {
    pub category: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingSloForecastQuery {
    pub fn matches(&self, item: &PendingSloForecast) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[PendingSloForecast]) -> Vec<PendingSloForecast> {
        let matched = collections::filter(items, |candidate: &PendingSloForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_slo_forecasts(
    store: Rc<dyn PendingSloForecastStore>,
    query: &PendingSloForecastQuery,
) -> Vec<PendingSloForecast> {
    query.apply(&store.list())
}
