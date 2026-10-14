use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient timeline forecasts by code.
pub trait LenientTimelineForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientTimelineForecast>;
    fn save(&self, item: &LenientTimelineForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientTimelineForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientTimelineForecastStore {
    pub items: RefCell<BTreeMap<String, LenientTimelineForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientTimelineForecastStore for MemoryLenientTimelineForecastStore {
    fn find(&self, code: &str) -> Option<LenientTimelineForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientTimelineForecast) {
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

    fn list(&self) -> Vec<LenientTimelineForecast> {
        let mut out: Vec<LenientTimelineForecast> = Vec::new();
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

pub fn new_memory_lenient_timeline_forecast_store() -> Rc<dyn LenientTimelineForecastStore> {
    Rc::new(MemoryLenientTimelineForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientTimelineForecastStore {
    pub inner: Rc<dyn LenientTimelineForecastStore>,
    pub cache: RefCell<cache::TtlCache<LenientTimelineForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientTimelineForecastStore for CachedLenientTimelineForecastStore {
    fn find(&self, code: &str) -> Option<LenientTimelineForecast> {
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

    fn save(&self, item: &LenientTimelineForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientTimelineForecast> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_timeline_forecast_store(
    inner: Rc<dyn LenientTimelineForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientTimelineForecastStore> {
    Rc::new(CachedLenientTimelineForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientTimelineForecast>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient timeline forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineForecastQuery {
    pub unit: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientTimelineForecastQuery {
    pub fn matches(&self, item: &LenientTimelineForecast) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientTimelineForecast]) -> Vec<LenientTimelineForecast> {
        let matched = collections::filter(items, |candidate: &LenientTimelineForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_timeline_forecasts(
    store: Rc<dyn LenientTimelineForecastStore>,
    query: &LenientTimelineForecastQuery,
) -> Vec<LenientTimelineForecast> {
    query.apply(&store.list())
}
