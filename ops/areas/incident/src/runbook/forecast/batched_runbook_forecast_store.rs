use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched runbook forecasts by code.
pub trait BatchedRunbookForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedRunbookForecast>;
    fn save(&self, item: &BatchedRunbookForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedRunbookForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedRunbookForecastStore {
    pub items: RefCell<BTreeMap<String, BatchedRunbookForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedRunbookForecastStore for MemoryBatchedRunbookForecastStore {
    fn find(&self, code: &str) -> Option<BatchedRunbookForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedRunbookForecast) {
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

    fn list(&self) -> Vec<BatchedRunbookForecast> {
        let mut out: Vec<BatchedRunbookForecast> = Vec::new();
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

pub fn new_memory_batched_runbook_forecast_store() -> Rc<dyn BatchedRunbookForecastStore> {
    Rc::new(MemoryBatchedRunbookForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedRunbookForecastStore {
    pub inner: Rc<dyn BatchedRunbookForecastStore>,
    pub cache: RefCell<cache::TtlCache<BatchedRunbookForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedRunbookForecastStore for CachedBatchedRunbookForecastStore {
    fn find(&self, code: &str) -> Option<BatchedRunbookForecast> {
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

    fn save(&self, item: &BatchedRunbookForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedRunbookForecast> {
        self.inner.list()
    }
}

pub fn new_cached_batched_runbook_forecast_store(
    inner: Rc<dyn BatchedRunbookForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedRunbookForecastStore> {
    Rc::new(CachedBatchedRunbookForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedRunbookForecast>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched runbook forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookForecastQuery {
    pub label: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedRunbookForecastQuery {
    pub fn matches(&self, item: &BatchedRunbookForecast) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedRunbookForecast]) -> Vec<BatchedRunbookForecast> {
        let matched = collections::filter(items, |candidate: &BatchedRunbookForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_runbook_forecasts(
    store: Rc<dyn BatchedRunbookForecastStore>,
    query: &BatchedRunbookForecastQuery,
) -> Vec<BatchedRunbookForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedRunbookForecastStore> = new_memory_batched_runbook_forecast_store();
        let item = new_batched_runbook_forecast("code-642", "label-938");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
