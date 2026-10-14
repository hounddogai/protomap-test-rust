use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient canary forecasts by code.
pub trait LenientCanaryForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientCanaryForecast>;
    fn save(&self, item: &LenientCanaryForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientCanaryForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientCanaryForecastStore {
    pub items: RefCell<BTreeMap<String, LenientCanaryForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientCanaryForecastStore for MemoryLenientCanaryForecastStore {
    fn find(&self, code: &str) -> Option<LenientCanaryForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientCanaryForecast) {
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

    fn list(&self) -> Vec<LenientCanaryForecast> {
        let mut out: Vec<LenientCanaryForecast> = Vec::new();
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

pub fn new_memory_lenient_canary_forecast_store() -> Rc<dyn LenientCanaryForecastStore> {
    Rc::new(MemoryLenientCanaryForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientCanaryForecastStore {
    pub inner: Rc<dyn LenientCanaryForecastStore>,
    pub cache: RefCell<cache::TtlCache<LenientCanaryForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientCanaryForecastStore for CachedLenientCanaryForecastStore {
    fn find(&self, code: &str) -> Option<LenientCanaryForecast> {
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

    fn save(&self, item: &LenientCanaryForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientCanaryForecast> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_canary_forecast_store(
    inner: Rc<dyn LenientCanaryForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientCanaryForecastStore> {
    Rc::new(CachedLenientCanaryForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientCanaryForecast>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient canary forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryForecastQuery {
    pub unit: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientCanaryForecastQuery {
    pub fn matches(&self, item: &LenientCanaryForecast) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[LenientCanaryForecast]) -> Vec<LenientCanaryForecast> {
        let matched = collections::filter(items, |candidate: &LenientCanaryForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_canary_forecasts(
    store: Rc<dyn LenientCanaryForecastStore>,
    query: &LenientCanaryForecastQuery,
) -> Vec<LenientCanaryForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientCanaryForecastStore> = new_memory_lenient_canary_forecast_store();
        let item = new_lenient_canary_forecast("code-507", "label-191");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
