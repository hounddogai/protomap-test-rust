use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient forecast estimates by code.
pub trait LenientForecastEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientForecastEstimate>;
    fn save(&self, item: &LenientForecastEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientForecastEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientForecastEstimateStore {
    pub items: RefCell<BTreeMap<String, LenientForecastEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientForecastEstimateStore for MemoryLenientForecastEstimateStore {
    fn find(&self, code: &str) -> Option<LenientForecastEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientForecastEstimate) {
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

    fn list(&self) -> Vec<LenientForecastEstimate> {
        let mut out: Vec<LenientForecastEstimate> = Vec::new();
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

pub fn new_memory_lenient_forecast_estimate_store() -> Rc<dyn LenientForecastEstimateStore> {
    Rc::new(MemoryLenientForecastEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientForecastEstimateStore {
    pub inner: Rc<dyn LenientForecastEstimateStore>,
    pub cache: RefCell<cache::TtlCache<LenientForecastEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientForecastEstimateStore for CachedLenientForecastEstimateStore {
    fn find(&self, code: &str) -> Option<LenientForecastEstimate> {
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

    fn save(&self, item: &LenientForecastEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientForecastEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_forecast_estimate_store(
    inner: Rc<dyn LenientForecastEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientForecastEstimateStore> {
    Rc::new(CachedLenientForecastEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientForecastEstimate>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient forecast estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastEstimateQuery {
    pub scope: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientForecastEstimateQuery {
    pub fn matches(&self, item: &LenientForecastEstimate) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientForecastEstimate]) -> Vec<LenientForecastEstimate> {
        let matched = collections::filter(items, |candidate: &LenientForecastEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_forecast_estimates(
    store: Rc<dyn LenientForecastEstimateStore>,
    query: &LenientForecastEstimateQuery,
) -> Vec<LenientForecastEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientForecastEstimateStore> = new_memory_lenient_forecast_estimate_store();
        let item = new_lenient_forecast_estimate("code-998", "label-329");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
