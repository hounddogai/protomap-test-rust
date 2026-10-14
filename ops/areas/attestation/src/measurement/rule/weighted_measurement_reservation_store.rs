use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted measurement reservations by code.
pub trait WeightedMeasurementReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedMeasurementReservation>;
    fn save(&self, item: &WeightedMeasurementReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedMeasurementReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedMeasurementReservationStore {
    pub items: RefCell<BTreeMap<String, WeightedMeasurementReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedMeasurementReservationStore for MemoryWeightedMeasurementReservationStore {
    fn find(&self, code: &str) -> Option<WeightedMeasurementReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedMeasurementReservation) {
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

    fn list(&self) -> Vec<WeightedMeasurementReservation> {
        let mut out: Vec<WeightedMeasurementReservation> = Vec::new();
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

pub fn new_memory_weighted_measurement_reservation_store() -> Rc<dyn WeightedMeasurementReservationStore> {
    Rc::new(MemoryWeightedMeasurementReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedMeasurementReservationStore {
    pub inner: Rc<dyn WeightedMeasurementReservationStore>,
    pub cache: RefCell<cache::TtlCache<WeightedMeasurementReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedMeasurementReservationStore for CachedWeightedMeasurementReservationStore {
    fn find(&self, code: &str) -> Option<WeightedMeasurementReservation> {
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

    fn save(&self, item: &WeightedMeasurementReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedMeasurementReservation> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_measurement_reservation_store(
    inner: Rc<dyn WeightedMeasurementReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedMeasurementReservationStore> {
    Rc::new(CachedWeightedMeasurementReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedMeasurementReservation>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted measurement reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementReservationQuery {
    pub country: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedMeasurementReservationQuery {
    pub fn matches(&self, item: &WeightedMeasurementReservation) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedMeasurementReservation]) -> Vec<WeightedMeasurementReservation> {
        let matched = collections::filter(items, |candidate: &WeightedMeasurementReservation| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_measurement_reservations(
    store: Rc<dyn WeightedMeasurementReservationStore>,
    query: &WeightedMeasurementReservationQuery,
) -> Vec<WeightedMeasurementReservation> {
    query.apply(&store.list())
}
