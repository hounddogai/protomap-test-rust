use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched forecast cycles by code.
pub trait BatchedForecastCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedForecastCycle>;
    fn save(&self, item: &BatchedForecastCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedForecastCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedForecastCycleStore {
    pub items: RefCell<BTreeMap<String, BatchedForecastCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedForecastCycleStore for MemoryBatchedForecastCycleStore {
    fn find(&self, code: &str) -> Option<BatchedForecastCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedForecastCycle) {
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

    fn list(&self) -> Vec<BatchedForecastCycle> {
        let mut out: Vec<BatchedForecastCycle> = Vec::new();
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

pub fn new_memory_batched_forecast_cycle_store() -> Rc<dyn BatchedForecastCycleStore> {
    Rc::new(MemoryBatchedForecastCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedForecastCycleStore {
    pub inner: Rc<dyn BatchedForecastCycleStore>,
    pub cache: RefCell<cache::TtlCache<BatchedForecastCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedForecastCycleStore for CachedBatchedForecastCycleStore {
    fn find(&self, code: &str) -> Option<BatchedForecastCycle> {
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

    fn save(&self, item: &BatchedForecastCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedForecastCycle> {
        self.inner.list()
    }
}

pub fn new_cached_batched_forecast_cycle_store(
    inner: Rc<dyn BatchedForecastCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedForecastCycleStore> {
    Rc::new(CachedBatchedForecastCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedForecastCycle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched forecast cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastCycleQuery {
    pub locale: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedForecastCycleQuery {
    pub fn matches(&self, item: &BatchedForecastCycle) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
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

    pub fn apply(&self, items: &[BatchedForecastCycle]) -> Vec<BatchedForecastCycle> {
        let matched = collections::filter(items, |candidate: &BatchedForecastCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_forecast_cycles(
    store: Rc<dyn BatchedForecastCycleStore>,
    query: &BatchedForecastCycleQuery,
) -> Vec<BatchedForecastCycle> {
    query.apply(&store.list())
}
