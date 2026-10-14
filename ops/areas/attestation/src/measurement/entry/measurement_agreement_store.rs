use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement agreements by code.
pub trait MeasurementAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementAgreement>;
    fn save(&self, item: &MeasurementAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementAgreementStore {
    pub items: RefCell<BTreeMap<String, MeasurementAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementAgreementStore for MemoryMeasurementAgreementStore {
    fn find(&self, code: &str) -> Option<MeasurementAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementAgreement) {
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

    fn list(&self) -> Vec<MeasurementAgreement> {
        let mut out: Vec<MeasurementAgreement> = Vec::new();
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

pub fn new_memory_measurement_agreement_store() -> Rc<dyn MeasurementAgreementStore> {
    Rc::new(MemoryMeasurementAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementAgreementStore {
    pub inner: Rc<dyn MeasurementAgreementStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementAgreementStore for CachedMeasurementAgreementStore {
    fn find(&self, code: &str) -> Option<MeasurementAgreement> {
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

    fn save(&self, item: &MeasurementAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_agreement_store(
    inner: Rc<dyn MeasurementAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementAgreementStore> {
    Rc::new(CachedMeasurementAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementAgreement>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAgreementQuery {
    pub segment: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementAgreementQuery {
    pub fn matches(&self, item: &MeasurementAgreement) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementAgreement]) -> Vec<MeasurementAgreement> {
        let matched = collections::filter(items, |candidate: &MeasurementAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_agreements(
    store: Rc<dyn MeasurementAgreementStore>,
    query: &MeasurementAgreementQuery,
) -> Vec<MeasurementAgreement> {
    query.apply(&store.list())
}
