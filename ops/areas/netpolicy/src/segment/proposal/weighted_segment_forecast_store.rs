use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted segment forecasts by code.
pub trait WeightedSegmentForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedSegmentForecast>;
    fn save(&self, item: &WeightedSegmentForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedSegmentForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedSegmentForecastStore {
    pub items: RefCell<BTreeMap<String, WeightedSegmentForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedSegmentForecastStore for MemoryWeightedSegmentForecastStore {
    fn find(&self, code: &str) -> Option<WeightedSegmentForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedSegmentForecast) {
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

    fn list(&self) -> Vec<WeightedSegmentForecast> {
        let mut out: Vec<WeightedSegmentForecast> = Vec::new();
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

pub fn new_memory_weighted_segment_forecast_store() -> Rc<dyn WeightedSegmentForecastStore> {
    Rc::new(MemoryWeightedSegmentForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedSegmentForecastStore {
    pub inner: Rc<dyn WeightedSegmentForecastStore>,
    pub cache: RefCell<cache::TtlCache<WeightedSegmentForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedSegmentForecastStore for CachedWeightedSegmentForecastStore {
    fn find(&self, code: &str) -> Option<WeightedSegmentForecast> {
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

    fn save(&self, item: &WeightedSegmentForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedSegmentForecast> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_segment_forecast_store(
    inner: Rc<dyn WeightedSegmentForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedSegmentForecastStore> {
    Rc::new(CachedWeightedSegmentForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedSegmentForecast>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted segment forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentForecastQuery {
    pub title: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedSegmentForecastQuery {
    pub fn matches(&self, item: &WeightedSegmentForecast) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedSegmentForecast]) -> Vec<WeightedSegmentForecast> {
        let matched = collections::filter(items, |candidate: &WeightedSegmentForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_segment_forecasts(
    store: Rc<dyn WeightedSegmentForecastStore>,
    query: &WeightedSegmentForecastQuery,
) -> Vec<WeightedSegmentForecast> {
    query.apply(&store.list())
}
