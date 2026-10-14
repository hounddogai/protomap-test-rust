use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched allowlist forecasts by code.
pub trait BatchedAllowlistForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedAllowlistForecast>;
    fn save(&self, item: &BatchedAllowlistForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedAllowlistForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedAllowlistForecastStore {
    pub items: RefCell<BTreeMap<String, BatchedAllowlistForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedAllowlistForecastStore for MemoryBatchedAllowlistForecastStore {
    fn find(&self, code: &str) -> Option<BatchedAllowlistForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedAllowlistForecast) {
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

    fn list(&self) -> Vec<BatchedAllowlistForecast> {
        let mut out: Vec<BatchedAllowlistForecast> = Vec::new();
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

pub fn new_memory_batched_allowlist_forecast_store() -> Rc<dyn BatchedAllowlistForecastStore> {
    Rc::new(MemoryBatchedAllowlistForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedAllowlistForecastStore {
    pub inner: Rc<dyn BatchedAllowlistForecastStore>,
    pub cache: RefCell<cache::TtlCache<BatchedAllowlistForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedAllowlistForecastStore for CachedBatchedAllowlistForecastStore {
    fn find(&self, code: &str) -> Option<BatchedAllowlistForecast> {
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

    fn save(&self, item: &BatchedAllowlistForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedAllowlistForecast> {
        self.inner.list()
    }
}

pub fn new_cached_batched_allowlist_forecast_store(
    inner: Rc<dyn BatchedAllowlistForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedAllowlistForecastStore> {
    Rc::new(CachedBatchedAllowlistForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedAllowlistForecast>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched allowlist forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistForecastQuery {
    pub namespace: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedAllowlistForecastQuery {
    pub fn matches(&self, item: &BatchedAllowlistForecast) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedAllowlistForecast]) -> Vec<BatchedAllowlistForecast> {
        let matched = collections::filter(items, |candidate: &BatchedAllowlistForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_allowlist_forecasts(
    store: Rc<dyn BatchedAllowlistForecastStore>,
    query: &BatchedAllowlistForecastQuery,
) -> Vec<BatchedAllowlistForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedAllowlistForecastStore> = new_memory_batched_allowlist_forecast_store();
        let item = new_batched_allowlist_forecast("code-933", "title-325");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
