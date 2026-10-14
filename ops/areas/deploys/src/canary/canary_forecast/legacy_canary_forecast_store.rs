use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy canary forecasts by code.
pub trait LegacyCanaryForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyCanaryForecast>;
    fn save(&self, item: &LegacyCanaryForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyCanaryForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyCanaryForecastStore {
    pub items: RefCell<BTreeMap<String, LegacyCanaryForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyCanaryForecastStore for MemoryLegacyCanaryForecastStore {
    fn find(&self, code: &str) -> Option<LegacyCanaryForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyCanaryForecast) {
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

    fn list(&self) -> Vec<LegacyCanaryForecast> {
        let mut out: Vec<LegacyCanaryForecast> = Vec::new();
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

pub fn new_memory_legacy_canary_forecast_store() -> Rc<dyn LegacyCanaryForecastStore> {
    Rc::new(MemoryLegacyCanaryForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyCanaryForecastStore {
    pub inner: Rc<dyn LegacyCanaryForecastStore>,
    pub cache: RefCell<cache::TtlCache<LegacyCanaryForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyCanaryForecastStore for CachedLegacyCanaryForecastStore {
    fn find(&self, code: &str) -> Option<LegacyCanaryForecast> {
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

    fn save(&self, item: &LegacyCanaryForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyCanaryForecast> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_canary_forecast_store(
    inner: Rc<dyn LegacyCanaryForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyCanaryForecastStore> {
    Rc::new(CachedLegacyCanaryForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyCanaryForecast>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy canary forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryForecastQuery {
    pub title: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyCanaryForecastQuery {
    pub fn matches(&self, item: &LegacyCanaryForecast) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyCanaryForecast]) -> Vec<LegacyCanaryForecast> {
        let matched = collections::filter(items, |candidate: &LegacyCanaryForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_canary_forecasts(
    store: Rc<dyn LegacyCanaryForecastStore>,
    query: &LegacyCanaryForecastQuery,
) -> Vec<LegacyCanaryForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyCanaryForecastStore> = new_memory_legacy_canary_forecast_store();
        let item = new_legacy_canary_forecast("code-649", "title-585");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
