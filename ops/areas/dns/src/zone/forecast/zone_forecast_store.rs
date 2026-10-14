use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps zone forecasts by code.
pub trait ZoneForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ZoneForecast>;
    fn save(&self, item: &ZoneForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ZoneForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryZoneForecastStore {
    pub items: RefCell<BTreeMap<String, ZoneForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl ZoneForecastStore for MemoryZoneForecastStore {
    fn find(&self, code: &str) -> Option<ZoneForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ZoneForecast) {
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

    fn list(&self) -> Vec<ZoneForecast> {
        let mut out: Vec<ZoneForecast> = Vec::new();
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

pub fn new_memory_zone_forecast_store() -> Rc<dyn ZoneForecastStore> {
    Rc::new(MemoryZoneForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedZoneForecastStore {
    pub inner: Rc<dyn ZoneForecastStore>,
    pub cache: RefCell<cache::TtlCache<ZoneForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ZoneForecastStore for CachedZoneForecastStore {
    fn find(&self, code: &str) -> Option<ZoneForecast> {
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

    fn save(&self, item: &ZoneForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ZoneForecast> {
        self.inner.list()
    }
}

pub fn new_cached_zone_forecast_store(
    inner: Rc<dyn ZoneForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ZoneForecastStore> {
    Rc::new(CachedZoneForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ZoneForecast>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects zone forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneForecastQuery {
    pub namespace: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ZoneForecastQuery {
    pub fn matches(&self, item: &ZoneForecast) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ZoneForecast]) -> Vec<ZoneForecast> {
        let matched = collections::filter(items, |candidate: &ZoneForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_zone_forecasts(store: Rc<dyn ZoneForecastStore>, query: &ZoneForecastQuery) -> Vec<ZoneForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ZoneForecastStore> = new_memory_zone_forecast_store();
        let item = new_zone_forecast("code-131", "name-397");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
