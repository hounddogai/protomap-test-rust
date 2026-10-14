use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast exceptions by code.
pub trait ForecastExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastException>;
    fn save(&self, item: &ForecastException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastExceptionStore {
    pub items: RefCell<BTreeMap<String, ForecastException>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastExceptionStore for MemoryForecastExceptionStore {
    fn find(&self, code: &str) -> Option<ForecastException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastException) {
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

    fn list(&self) -> Vec<ForecastException> {
        let mut out: Vec<ForecastException> = Vec::new();
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

pub fn new_memory_forecast_exception_store() -> Rc<dyn ForecastExceptionStore> {
    Rc::new(MemoryForecastExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastExceptionStore {
    pub inner: Rc<dyn ForecastExceptionStore>,
    pub cache: RefCell<cache::TtlCache<ForecastException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastExceptionStore for CachedForecastExceptionStore {
    fn find(&self, code: &str) -> Option<ForecastException> {
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

    fn save(&self, item: &ForecastException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastException> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_exception_store(
    inner: Rc<dyn ForecastExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastExceptionStore> {
    Rc::new(CachedForecastExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastException>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastExceptionQuery {
    pub locale: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastExceptionQuery {
    pub fn matches(&self, item: &ForecastException) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
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

    pub fn apply(&self, items: &[ForecastException]) -> Vec<ForecastException> {
        let matched = collections::filter(items, |candidate: &ForecastException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_exceptions(
    store: Rc<dyn ForecastExceptionStore>,
    query: &ForecastExceptionQuery,
) -> Vec<ForecastException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ForecastExceptionStore> = new_memory_forecast_exception_store();
        let item = new_forecast_exception("code-242", "name-636");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
