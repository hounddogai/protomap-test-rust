use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending page forecasts by code.
pub trait PendingPageForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingPageForecast>;
    fn save(&self, item: &PendingPageForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingPageForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingPageForecastStore {
    pub items: RefCell<BTreeMap<String, PendingPageForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingPageForecastStore for MemoryPendingPageForecastStore {
    fn find(&self, code: &str) -> Option<PendingPageForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingPageForecast) {
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

    fn list(&self) -> Vec<PendingPageForecast> {
        let mut out: Vec<PendingPageForecast> = Vec::new();
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

pub fn new_memory_pending_page_forecast_store() -> Rc<dyn PendingPageForecastStore> {
    Rc::new(MemoryPendingPageForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingPageForecastStore {
    pub inner: Rc<dyn PendingPageForecastStore>,
    pub cache: RefCell<cache::TtlCache<PendingPageForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingPageForecastStore for CachedPendingPageForecastStore {
    fn find(&self, code: &str) -> Option<PendingPageForecast> {
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

    fn save(&self, item: &PendingPageForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingPageForecast> {
        self.inner.list()
    }
}

pub fn new_cached_pending_page_forecast_store(
    inner: Rc<dyn PendingPageForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingPageForecastStore> {
    Rc::new(CachedPendingPageForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingPageForecast>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending page forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageForecastQuery {
    pub segment: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingPageForecastQuery {
    pub fn matches(&self, item: &PendingPageForecast) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingPageForecast]) -> Vec<PendingPageForecast> {
        let matched = collections::filter(items, |candidate: &PendingPageForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_page_forecasts(
    store: Rc<dyn PendingPageForecastStore>,
    query: &PendingPageForecastQuery,
) -> Vec<PendingPageForecast> {
    query.apply(&store.list())
}
