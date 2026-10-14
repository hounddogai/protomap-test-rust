use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted budget burn forecasts by code.
pub trait WeightedBudgetBurnForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedBudgetBurnForecast>;
    fn save(&self, item: &WeightedBudgetBurnForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedBudgetBurnForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedBudgetBurnForecastStore {
    pub items: RefCell<BTreeMap<String, WeightedBudgetBurnForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedBudgetBurnForecastStore for MemoryWeightedBudgetBurnForecastStore {
    fn find(&self, code: &str) -> Option<WeightedBudgetBurnForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedBudgetBurnForecast) {
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

    fn list(&self) -> Vec<WeightedBudgetBurnForecast> {
        let mut out: Vec<WeightedBudgetBurnForecast> = Vec::new();
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

pub fn new_memory_weighted_budget_burn_forecast_store() -> Rc<dyn WeightedBudgetBurnForecastStore> {
    Rc::new(MemoryWeightedBudgetBurnForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedBudgetBurnForecastStore {
    pub inner: Rc<dyn WeightedBudgetBurnForecastStore>,
    pub cache: RefCell<cache::TtlCache<WeightedBudgetBurnForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedBudgetBurnForecastStore for CachedWeightedBudgetBurnForecastStore {
    fn find(&self, code: &str) -> Option<WeightedBudgetBurnForecast> {
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

    fn save(&self, item: &WeightedBudgetBurnForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedBudgetBurnForecast> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_budget_burn_forecast_store(
    inner: Rc<dyn WeightedBudgetBurnForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedBudgetBurnForecastStore> {
    Rc::new(CachedWeightedBudgetBurnForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedBudgetBurnForecast>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted budget burn forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnForecastQuery {
    pub namespace: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedBudgetBurnForecastQuery {
    pub fn matches(&self, item: &WeightedBudgetBurnForecast) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedBudgetBurnForecast]) -> Vec<WeightedBudgetBurnForecast> {
        let matched = collections::filter(items, |candidate: &WeightedBudgetBurnForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_budget_burn_forecasts(
    store: Rc<dyn WeightedBudgetBurnForecastStore>,
    query: &WeightedBudgetBurnForecastQuery,
) -> Vec<WeightedBudgetBurnForecast> {
    query.apply(&store.list())
}
