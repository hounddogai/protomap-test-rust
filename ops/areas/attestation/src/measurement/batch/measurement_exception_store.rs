use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement exceptions by code.
pub trait MeasurementExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementException>;
    fn save(&self, item: &MeasurementException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementExceptionStore {
    pub items: RefCell<BTreeMap<String, MeasurementException>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementExceptionStore for MemoryMeasurementExceptionStore {
    fn find(&self, code: &str) -> Option<MeasurementException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementException) {
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

    fn list(&self) -> Vec<MeasurementException> {
        let mut out: Vec<MeasurementException> = Vec::new();
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

pub fn new_memory_measurement_exception_store() -> Rc<dyn MeasurementExceptionStore> {
    Rc::new(MemoryMeasurementExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementExceptionStore {
    pub inner: Rc<dyn MeasurementExceptionStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementExceptionStore for CachedMeasurementExceptionStore {
    fn find(&self, code: &str) -> Option<MeasurementException> {
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

    fn save(&self, item: &MeasurementException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementException> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_exception_store(
    inner: Rc<dyn MeasurementExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementExceptionStore> {
    Rc::new(CachedMeasurementExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementException>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementExceptionQuery {
    pub namespace: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementExceptionQuery {
    pub fn matches(&self, item: &MeasurementException) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[MeasurementException]) -> Vec<MeasurementException> {
        let matched = collections::filter(items, |candidate: &MeasurementException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_exceptions(
    store: Rc<dyn MeasurementExceptionStore>,
    query: &MeasurementExceptionQuery,
) -> Vec<MeasurementException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MeasurementExceptionStore> = new_memory_measurement_exception_store();
        let item = new_measurement_exception("code-517", "name-957");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
