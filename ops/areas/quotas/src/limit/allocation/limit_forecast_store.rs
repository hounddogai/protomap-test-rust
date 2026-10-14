use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps limit forecasts by code.
pub trait LimitForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LimitForecast>;
    fn save(&self, item: &LimitForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LimitForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLimitForecastStore {
    pub items: RefCell<BTreeMap<String, LimitForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl LimitForecastStore for MemoryLimitForecastStore {
    fn find(&self, code: &str) -> Option<LimitForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LimitForecast) {
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

    fn list(&self) -> Vec<LimitForecast> {
        let mut out: Vec<LimitForecast> = Vec::new();
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

pub fn new_memory_limit_forecast_store() -> Rc<dyn LimitForecastStore> {
    Rc::new(MemoryLimitForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLimitForecastStore {
    pub inner: Rc<dyn LimitForecastStore>,
    pub cache: RefCell<cache::TtlCache<LimitForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LimitForecastStore for CachedLimitForecastStore {
    fn find(&self, code: &str) -> Option<LimitForecast> {
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

    fn save(&self, item: &LimitForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LimitForecast> {
        self.inner.list()
    }
}

pub fn new_cached_limit_forecast_store(
    inner: Rc<dyn LimitForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LimitForecastStore> {
    Rc::new(CachedLimitForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LimitForecast>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects limit forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitForecastQuery {
    pub label: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LimitForecastQuery {
    pub fn matches(&self, item: &LimitForecast) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LimitForecast]) -> Vec<LimitForecast> {
        let matched = collections::filter(items, |candidate: &LimitForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_limit_forecasts(store: Rc<dyn LimitForecastStore>, query: &LimitForecastQuery) -> Vec<LimitForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LimitForecastStore> = new_memory_limit_forecast_store();
        let item = new_limit_forecast("code-369", "label-215");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
