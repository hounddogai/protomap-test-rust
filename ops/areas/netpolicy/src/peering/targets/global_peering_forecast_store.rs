use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global peering forecasts by code.
pub trait GlobalPeeringForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalPeeringForecast>;
    fn save(&self, item: &GlobalPeeringForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalPeeringForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalPeeringForecastStore {
    pub items: RefCell<BTreeMap<String, GlobalPeeringForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalPeeringForecastStore for MemoryGlobalPeeringForecastStore {
    fn find(&self, code: &str) -> Option<GlobalPeeringForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalPeeringForecast) {
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

    fn list(&self) -> Vec<GlobalPeeringForecast> {
        let mut out: Vec<GlobalPeeringForecast> = Vec::new();
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

pub fn new_memory_global_peering_forecast_store() -> Rc<dyn GlobalPeeringForecastStore> {
    Rc::new(MemoryGlobalPeeringForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalPeeringForecastStore {
    pub inner: Rc<dyn GlobalPeeringForecastStore>,
    pub cache: RefCell<cache::TtlCache<GlobalPeeringForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalPeeringForecastStore for CachedGlobalPeeringForecastStore {
    fn find(&self, code: &str) -> Option<GlobalPeeringForecast> {
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

    fn save(&self, item: &GlobalPeeringForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalPeeringForecast> {
        self.inner.list()
    }
}

pub fn new_cached_global_peering_forecast_store(
    inner: Rc<dyn GlobalPeeringForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalPeeringForecastStore> {
    Rc::new(CachedGlobalPeeringForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalPeeringForecast>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global peering forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringForecastQuery {
    pub title: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalPeeringForecastQuery {
    pub fn matches(&self, item: &GlobalPeeringForecast) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[GlobalPeeringForecast]) -> Vec<GlobalPeeringForecast> {
        let matched = collections::filter(items, |candidate: &GlobalPeeringForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_peering_forecasts(
    store: Rc<dyn GlobalPeeringForecastStore>,
    query: &GlobalPeeringForecastQuery,
) -> Vec<GlobalPeeringForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalPeeringForecastStore> = new_memory_global_peering_forecast_store();
        let item = new_global_peering_forecast("code-263", "title-829");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
