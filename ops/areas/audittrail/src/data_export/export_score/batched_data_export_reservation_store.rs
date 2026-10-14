use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched data export reservations by code.
pub trait BatchedDataExportReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedDataExportReservation>;
    fn save(&self, item: &BatchedDataExportReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedDataExportReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedDataExportReservationStore {
    pub items: RefCell<BTreeMap<String, BatchedDataExportReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedDataExportReservationStore for MemoryBatchedDataExportReservationStore {
    fn find(&self, code: &str) -> Option<BatchedDataExportReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedDataExportReservation) {
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

    fn list(&self) -> Vec<BatchedDataExportReservation> {
        let mut out: Vec<BatchedDataExportReservation> = Vec::new();
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

pub fn new_memory_batched_data_export_reservation_store() -> Rc<dyn BatchedDataExportReservationStore> {
    Rc::new(MemoryBatchedDataExportReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedDataExportReservationStore {
    pub inner: Rc<dyn BatchedDataExportReservationStore>,
    pub cache: RefCell<cache::TtlCache<BatchedDataExportReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedDataExportReservationStore for CachedBatchedDataExportReservationStore {
    fn find(&self, code: &str) -> Option<BatchedDataExportReservation> {
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

    fn save(&self, item: &BatchedDataExportReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedDataExportReservation> {
        self.inner.list()
    }
}

pub fn new_cached_batched_data_export_reservation_store(
    inner: Rc<dyn BatchedDataExportReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedDataExportReservationStore> {
    Rc::new(CachedBatchedDataExportReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedDataExportReservation>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched data export reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDataExportReservationQuery {
    pub title: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedDataExportReservationQuery {
    pub fn matches(&self, item: &BatchedDataExportReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedDataExportReservation]) -> Vec<BatchedDataExportReservation> {
        let matched = collections::filter(items, |candidate: &BatchedDataExportReservation| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_data_export_reservations(
    store: Rc<dyn BatchedDataExportReservationStore>,
    query: &BatchedDataExportReservationQuery,
) -> Vec<BatchedDataExportReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedDataExportReservationStore> = new_memory_batched_data_export_reservation_store();
        let item = new_batched_data_export_reservation("code-8", "title-397");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
