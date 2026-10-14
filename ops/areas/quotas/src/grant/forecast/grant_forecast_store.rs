use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps grant forecasts by code.
pub trait GrantForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GrantForecast>;
    fn save(&self, item: &GrantForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GrantForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGrantForecastStore {
    pub items: RefCell<BTreeMap<String, GrantForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl GrantForecastStore for MemoryGrantForecastStore {
    fn find(&self, code: &str) -> Option<GrantForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GrantForecast) {
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

    fn list(&self) -> Vec<GrantForecast> {
        let mut out: Vec<GrantForecast> = Vec::new();
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

pub fn new_memory_grant_forecast_store() -> Rc<dyn GrantForecastStore> {
    Rc::new(MemoryGrantForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGrantForecastStore {
    pub inner: Rc<dyn GrantForecastStore>,
    pub cache: RefCell<cache::TtlCache<GrantForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GrantForecastStore for CachedGrantForecastStore {
    fn find(&self, code: &str) -> Option<GrantForecast> {
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

    fn save(&self, item: &GrantForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GrantForecast> {
        self.inner.list()
    }
}

pub fn new_cached_grant_forecast_store(
    inner: Rc<dyn GrantForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GrantForecastStore> {
    Rc::new(CachedGrantForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GrantForecast>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects grant forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantForecastQuery {
    pub namespace: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GrantForecastQuery {
    pub fn matches(&self, item: &GrantForecast) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GrantForecast]) -> Vec<GrantForecast> {
        let matched = collections::filter(items, |candidate: &GrantForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_grant_forecasts(store: Rc<dyn GrantForecastStore>, query: &GrantForecastQuery) -> Vec<GrantForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GrantForecastStore> = new_memory_grant_forecast_store();
        let item = new_grant_forecast("code-214", "name-614");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
