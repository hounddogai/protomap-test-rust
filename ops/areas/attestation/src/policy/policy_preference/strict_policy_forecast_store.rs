use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict policy forecasts by code.
pub trait StrictPolicyForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictPolicyForecast>;
    fn save(&self, item: &StrictPolicyForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictPolicyForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictPolicyForecastStore {
    pub items: RefCell<BTreeMap<String, StrictPolicyForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictPolicyForecastStore for MemoryStrictPolicyForecastStore {
    fn find(&self, code: &str) -> Option<StrictPolicyForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictPolicyForecast) {
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

    fn list(&self) -> Vec<StrictPolicyForecast> {
        let mut out: Vec<StrictPolicyForecast> = Vec::new();
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

pub fn new_memory_strict_policy_forecast_store() -> Rc<dyn StrictPolicyForecastStore> {
    Rc::new(MemoryStrictPolicyForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictPolicyForecastStore {
    pub inner: Rc<dyn StrictPolicyForecastStore>,
    pub cache: RefCell<cache::TtlCache<StrictPolicyForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictPolicyForecastStore for CachedStrictPolicyForecastStore {
    fn find(&self, code: &str) -> Option<StrictPolicyForecast> {
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

    fn save(&self, item: &StrictPolicyForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictPolicyForecast> {
        self.inner.list()
    }
}

pub fn new_cached_strict_policy_forecast_store(
    inner: Rc<dyn StrictPolicyForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictPolicyForecastStore> {
    Rc::new(CachedStrictPolicyForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictPolicyForecast>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict policy forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyForecastQuery {
    pub label: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictPolicyForecastQuery {
    pub fn matches(&self, item: &StrictPolicyForecast) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[StrictPolicyForecast]) -> Vec<StrictPolicyForecast> {
        let matched = collections::filter(items, |candidate: &StrictPolicyForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_policy_forecasts(
    store: Rc<dyn StrictPolicyForecastStore>,
    query: &StrictPolicyForecastQuery,
) -> Vec<StrictPolicyForecast> {
    query.apply(&store.list())
}
