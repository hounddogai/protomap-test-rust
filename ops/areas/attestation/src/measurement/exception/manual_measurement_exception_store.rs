use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual measurement exceptions by code.
pub trait ManualMeasurementExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualMeasurementException>;
    fn save(&self, item: &ManualMeasurementException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualMeasurementException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualMeasurementExceptionStore {
    pub items: RefCell<BTreeMap<String, ManualMeasurementException>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualMeasurementExceptionStore for MemoryManualMeasurementExceptionStore {
    fn find(&self, code: &str) -> Option<ManualMeasurementException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualMeasurementException) {
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

    fn list(&self) -> Vec<ManualMeasurementException> {
        let mut out: Vec<ManualMeasurementException> = Vec::new();
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

pub fn new_memory_manual_measurement_exception_store() -> Rc<dyn ManualMeasurementExceptionStore> {
    Rc::new(MemoryManualMeasurementExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualMeasurementExceptionStore {
    pub inner: Rc<dyn ManualMeasurementExceptionStore>,
    pub cache: RefCell<cache::TtlCache<ManualMeasurementException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualMeasurementExceptionStore for CachedManualMeasurementExceptionStore {
    fn find(&self, code: &str) -> Option<ManualMeasurementException> {
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

    fn save(&self, item: &ManualMeasurementException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualMeasurementException> {
        self.inner.list()
    }
}

pub fn new_cached_manual_measurement_exception_store(
    inner: Rc<dyn ManualMeasurementExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualMeasurementExceptionStore> {
    Rc::new(CachedManualMeasurementExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualMeasurementException>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual measurement exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementExceptionQuery {
    pub country: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualMeasurementExceptionQuery {
    pub fn matches(&self, item: &ManualMeasurementException) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualMeasurementException]) -> Vec<ManualMeasurementException> {
        let matched = collections::filter(items, |candidate: &ManualMeasurementException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_measurement_exceptions(
    store: Rc<dyn ManualMeasurementExceptionStore>,
    query: &ManualMeasurementExceptionQuery,
) -> Vec<ManualMeasurementException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualMeasurementExceptionStore> = new_memory_manual_measurement_exception_store();
        let item = new_manual_measurement_exception("code-434", "title-889");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
