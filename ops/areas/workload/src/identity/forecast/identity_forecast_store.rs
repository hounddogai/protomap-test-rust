use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity forecasts by code.
pub trait IdentityForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityForecast>;
    fn save(&self, item: &IdentityForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityForecastStore {
    pub items: RefCell<BTreeMap<String, IdentityForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityForecastStore for MemoryIdentityForecastStore {
    fn find(&self, code: &str) -> Option<IdentityForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityForecast) {
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

    fn list(&self) -> Vec<IdentityForecast> {
        let mut out: Vec<IdentityForecast> = Vec::new();
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

pub fn new_memory_identity_forecast_store() -> Rc<dyn IdentityForecastStore> {
    Rc::new(MemoryIdentityForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityForecastStore {
    pub inner: Rc<dyn IdentityForecastStore>,
    pub cache: RefCell<cache::TtlCache<IdentityForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityForecastStore for CachedIdentityForecastStore {
    fn find(&self, code: &str) -> Option<IdentityForecast> {
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

    fn save(&self, item: &IdentityForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityForecast> {
        self.inner.list()
    }
}

pub fn new_cached_identity_forecast_store(
    inner: Rc<dyn IdentityForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityForecastStore> {
    Rc::new(CachedIdentityForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityForecast>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityForecastQuery {
    pub label: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityForecastQuery {
    pub fn matches(&self, item: &IdentityForecast) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IdentityForecast]) -> Vec<IdentityForecast> {
        let matched = collections::filter(items, |candidate: &IdentityForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_forecasts(
    store: Rc<dyn IdentityForecastStore>,
    query: &IdentityForecastQuery,
) -> Vec<IdentityForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IdentityForecastStore> = new_memory_identity_forecast_store();
        let item = new_identity_forecast("code-505", "label-932");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
