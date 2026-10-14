use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering forecasts by code.
pub trait PeeringForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringForecast>;
    fn save(&self, item: &PeeringForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringForecastStore {
    pub items: RefCell<BTreeMap<String, PeeringForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringForecastStore for MemoryPeeringForecastStore {
    fn find(&self, code: &str) -> Option<PeeringForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringForecast) {
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

    fn list(&self) -> Vec<PeeringForecast> {
        let mut out: Vec<PeeringForecast> = Vec::new();
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

pub fn new_memory_peering_forecast_store() -> Rc<dyn PeeringForecastStore> {
    Rc::new(MemoryPeeringForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringForecastStore {
    pub inner: Rc<dyn PeeringForecastStore>,
    pub cache: RefCell<cache::TtlCache<PeeringForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringForecastStore for CachedPeeringForecastStore {
    fn find(&self, code: &str) -> Option<PeeringForecast> {
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

    fn save(&self, item: &PeeringForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringForecast> {
        self.inner.list()
    }
}

pub fn new_cached_peering_forecast_store(
    inner: Rc<dyn PeeringForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringForecastStore> {
    Rc::new(CachedPeeringForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringForecast>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringForecastQuery {
    pub title: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringForecastQuery {
    pub fn matches(&self, item: &PeeringForecast) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PeeringForecast]) -> Vec<PeeringForecast> {
        let matched = collections::filter(items, |candidate: &PeeringForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_forecasts(
    store: Rc<dyn PeeringForecastStore>,
    query: &PeeringForecastQuery,
) -> Vec<PeeringForecast> {
    query.apply(&store.list())
}
