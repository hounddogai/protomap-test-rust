use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement requests by code.
pub trait MeasurementRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementRequest>;
    fn save(&self, item: &MeasurementRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementRequestStore {
    pub items: RefCell<BTreeMap<String, MeasurementRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementRequestStore for MemoryMeasurementRequestStore {
    fn find(&self, code: &str) -> Option<MeasurementRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementRequest) {
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

    fn list(&self) -> Vec<MeasurementRequest> {
        let mut out: Vec<MeasurementRequest> = Vec::new();
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

pub fn new_memory_measurement_request_store() -> Rc<dyn MeasurementRequestStore> {
    Rc::new(MemoryMeasurementRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementRequestStore {
    pub inner: Rc<dyn MeasurementRequestStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementRequestStore for CachedMeasurementRequestStore {
    fn find(&self, code: &str) -> Option<MeasurementRequest> {
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

    fn save(&self, item: &MeasurementRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementRequest> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_request_store(
    inner: Rc<dyn MeasurementRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementRequestStore> {
    Rc::new(CachedMeasurementRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementRequest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRequestQuery {
    pub country: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementRequestQuery {
    pub fn matches(&self, item: &MeasurementRequest) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementRequest]) -> Vec<MeasurementRequest> {
        let matched = collections::filter(items, |candidate: &MeasurementRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_requests(
    store: Rc<dyn MeasurementRequestStore>,
    query: &MeasurementRequestQuery,
) -> Vec<MeasurementRequest> {
    query.apply(&store.list())
}
