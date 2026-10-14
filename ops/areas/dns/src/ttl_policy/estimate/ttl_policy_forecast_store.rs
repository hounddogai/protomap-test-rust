use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps ttl policy forecasts by code.
pub trait TtlPolicyForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TtlPolicyForecast>;
    fn save(&self, item: &TtlPolicyForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TtlPolicyForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTtlPolicyForecastStore {
    pub items: RefCell<BTreeMap<String, TtlPolicyForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl TtlPolicyForecastStore for MemoryTtlPolicyForecastStore {
    fn find(&self, code: &str) -> Option<TtlPolicyForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TtlPolicyForecast) {
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

    fn list(&self) -> Vec<TtlPolicyForecast> {
        let mut out: Vec<TtlPolicyForecast> = Vec::new();
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

pub fn new_memory_ttl_policy_forecast_store() -> Rc<dyn TtlPolicyForecastStore> {
    Rc::new(MemoryTtlPolicyForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTtlPolicyForecastStore {
    pub inner: Rc<dyn TtlPolicyForecastStore>,
    pub cache: RefCell<cache::TtlCache<TtlPolicyForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TtlPolicyForecastStore for CachedTtlPolicyForecastStore {
    fn find(&self, code: &str) -> Option<TtlPolicyForecast> {
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

    fn save(&self, item: &TtlPolicyForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TtlPolicyForecast> {
        self.inner.list()
    }
}

pub fn new_cached_ttl_policy_forecast_store(
    inner: Rc<dyn TtlPolicyForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TtlPolicyForecastStore> {
    Rc::new(CachedTtlPolicyForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TtlPolicyForecast>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects ttl policy forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyForecastQuery {
    pub unit: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TtlPolicyForecastQuery {
    pub fn matches(&self, item: &TtlPolicyForecast) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TtlPolicyForecast]) -> Vec<TtlPolicyForecast> {
        let matched = collections::filter(items, |candidate: &TtlPolicyForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_ttl_policy_forecasts(
    store: Rc<dyn TtlPolicyForecastStore>,
    query: &TtlPolicyForecastQuery,
) -> Vec<TtlPolicyForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TtlPolicyForecastStore> = new_memory_ttl_policy_forecast_store();
        let item = new_ttl_policy_forecast("code-66", "label-863");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
