use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual forecast exceptions by code.
pub trait ManualForecastExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualForecastException>;
    fn save(&self, item: &ManualForecastException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualForecastException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualForecastExceptionStore {
    pub items: RefCell<BTreeMap<String, ManualForecastException>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualForecastExceptionStore for MemoryManualForecastExceptionStore {
    fn find(&self, code: &str) -> Option<ManualForecastException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualForecastException) {
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

    fn list(&self) -> Vec<ManualForecastException> {
        let mut out: Vec<ManualForecastException> = Vec::new();
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

pub fn new_memory_manual_forecast_exception_store() -> Rc<dyn ManualForecastExceptionStore> {
    Rc::new(MemoryManualForecastExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualForecastExceptionStore {
    pub inner: Rc<dyn ManualForecastExceptionStore>,
    pub cache: RefCell<cache::TtlCache<ManualForecastException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualForecastExceptionStore for CachedManualForecastExceptionStore {
    fn find(&self, code: &str) -> Option<ManualForecastException> {
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

    fn save(&self, item: &ManualForecastException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualForecastException> {
        self.inner.list()
    }
}

pub fn new_cached_manual_forecast_exception_store(
    inner: Rc<dyn ManualForecastExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualForecastExceptionStore> {
    Rc::new(CachedManualForecastExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualForecastException>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual forecast exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastExceptionQuery {
    pub label: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualForecastExceptionQuery {
    pub fn matches(&self, item: &ManualForecastException) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualForecastException]) -> Vec<ManualForecastException> {
        let matched = collections::filter(items, |candidate: &ManualForecastException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_forecast_exceptions(
    store: Rc<dyn ManualForecastExceptionStore>,
    query: &ManualForecastExceptionQuery,
) -> Vec<ManualForecastException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualForecastExceptionStore> = new_memory_manual_forecast_exception_store();
        let item = new_manual_forecast_exception("code-141", "label-714");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
