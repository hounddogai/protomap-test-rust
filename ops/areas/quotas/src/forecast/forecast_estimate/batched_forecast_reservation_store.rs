use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched forecast reservations by code.
pub trait BatchedForecastReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedForecastReservation>;
    fn save(&self, item: &BatchedForecastReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedForecastReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedForecastReservationStore {
    pub items: RefCell<BTreeMap<String, BatchedForecastReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedForecastReservationStore for MemoryBatchedForecastReservationStore {
    fn find(&self, code: &str) -> Option<BatchedForecastReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedForecastReservation) {
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

    fn list(&self) -> Vec<BatchedForecastReservation> {
        let mut out: Vec<BatchedForecastReservation> = Vec::new();
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

pub fn new_memory_batched_forecast_reservation_store() -> Rc<dyn BatchedForecastReservationStore> {
    Rc::new(MemoryBatchedForecastReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedForecastReservationStore {
    pub inner: Rc<dyn BatchedForecastReservationStore>,
    pub cache: RefCell<cache::TtlCache<BatchedForecastReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedForecastReservationStore for CachedBatchedForecastReservationStore {
    fn find(&self, code: &str) -> Option<BatchedForecastReservation> {
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

    fn save(&self, item: &BatchedForecastReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedForecastReservation> {
        self.inner.list()
    }
}

pub fn new_cached_batched_forecast_reservation_store(
    inner: Rc<dyn BatchedForecastReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedForecastReservationStore> {
    Rc::new(CachedBatchedForecastReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedForecastReservation>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched forecast reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReservationQuery {
    pub category: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedForecastReservationQuery {
    pub fn matches(&self, item: &BatchedForecastReservation) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[BatchedForecastReservation]) -> Vec<BatchedForecastReservation> {
        let matched = collections::filter(items, |candidate: &BatchedForecastReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_forecast_reservations(
    store: Rc<dyn BatchedForecastReservationStore>,
    query: &BatchedForecastReservationQuery,
) -> Vec<BatchedForecastReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedForecastReservationStore> = new_memory_batched_forecast_reservation_store();
        let item = new_batched_forecast_reservation("code-774", "label-786");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
