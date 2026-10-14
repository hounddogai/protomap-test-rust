use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding forecasts by code.
pub trait BindingForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingForecast>;
    fn save(&self, item: &BindingForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingForecastStore {
    pub items: RefCell<BTreeMap<String, BindingForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingForecastStore for MemoryBindingForecastStore {
    fn find(&self, code: &str) -> Option<BindingForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingForecast) {
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

    fn list(&self) -> Vec<BindingForecast> {
        let mut out: Vec<BindingForecast> = Vec::new();
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

pub fn new_memory_binding_forecast_store() -> Rc<dyn BindingForecastStore> {
    Rc::new(MemoryBindingForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingForecastStore {
    pub inner: Rc<dyn BindingForecastStore>,
    pub cache: RefCell<cache::TtlCache<BindingForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingForecastStore for CachedBindingForecastStore {
    fn find(&self, code: &str) -> Option<BindingForecast> {
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

    fn save(&self, item: &BindingForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingForecast> {
        self.inner.list()
    }
}

pub fn new_cached_binding_forecast_store(
    inner: Rc<dyn BindingForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingForecastStore> {
    Rc::new(CachedBindingForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingForecast>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingForecastQuery {
    pub name: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingForecastQuery {
    pub fn matches(&self, item: &BindingForecast) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingForecast]) -> Vec<BindingForecast> {
        let matched = collections::filter(items, |candidate: &BindingForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_forecasts(
    store: Rc<dyn BindingForecastStore>,
    query: &BindingForecastQuery,
) -> Vec<BindingForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingForecastStore> = new_memory_binding_forecast_store();
        let item = new_binding_forecast("code-805", "name-798");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
