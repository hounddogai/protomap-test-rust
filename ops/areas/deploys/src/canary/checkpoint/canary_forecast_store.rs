use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps canary forecasts by code.
pub trait CanaryForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CanaryForecast>;
    fn save(&self, item: &CanaryForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CanaryForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCanaryForecastStore {
    pub items: RefCell<BTreeMap<String, CanaryForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl CanaryForecastStore for MemoryCanaryForecastStore {
    fn find(&self, code: &str) -> Option<CanaryForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CanaryForecast) {
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

    fn list(&self) -> Vec<CanaryForecast> {
        let mut out: Vec<CanaryForecast> = Vec::new();
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

pub fn new_memory_canary_forecast_store() -> Rc<dyn CanaryForecastStore> {
    Rc::new(MemoryCanaryForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCanaryForecastStore {
    pub inner: Rc<dyn CanaryForecastStore>,
    pub cache: RefCell<cache::TtlCache<CanaryForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CanaryForecastStore for CachedCanaryForecastStore {
    fn find(&self, code: &str) -> Option<CanaryForecast> {
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

    fn save(&self, item: &CanaryForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CanaryForecast> {
        self.inner.list()
    }
}

pub fn new_cached_canary_forecast_store(
    inner: Rc<dyn CanaryForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CanaryForecastStore> {
    Rc::new(CachedCanaryForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CanaryForecast>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects canary forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecastQuery {
    pub label: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CanaryForecastQuery {
    pub fn matches(&self, item: &CanaryForecast) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CanaryForecast]) -> Vec<CanaryForecast> {
        let matched = collections::filter(items, |candidate: &CanaryForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_canary_forecasts(store: Rc<dyn CanaryForecastStore>, query: &CanaryForecastQuery) -> Vec<CanaryForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CanaryForecastStore> = new_memory_canary_forecast_store();
        let item = new_canary_forecast("code-446", "label-812");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
