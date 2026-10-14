use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched escalation reservations by code.
pub trait BatchedEscalationReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedEscalationReservation>;
    fn save(&self, item: &BatchedEscalationReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedEscalationReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedEscalationReservationStore {
    pub items: RefCell<BTreeMap<String, BatchedEscalationReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedEscalationReservationStore for MemoryBatchedEscalationReservationStore {
    fn find(&self, code: &str) -> Option<BatchedEscalationReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedEscalationReservation) {
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

    fn list(&self) -> Vec<BatchedEscalationReservation> {
        let mut out: Vec<BatchedEscalationReservation> = Vec::new();
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

pub fn new_memory_batched_escalation_reservation_store() -> Rc<dyn BatchedEscalationReservationStore> {
    Rc::new(MemoryBatchedEscalationReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedEscalationReservationStore {
    pub inner: Rc<dyn BatchedEscalationReservationStore>,
    pub cache: RefCell<cache::TtlCache<BatchedEscalationReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedEscalationReservationStore for CachedBatchedEscalationReservationStore {
    fn find(&self, code: &str) -> Option<BatchedEscalationReservation> {
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

    fn save(&self, item: &BatchedEscalationReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedEscalationReservation> {
        self.inner.list()
    }
}

pub fn new_cached_batched_escalation_reservation_store(
    inner: Rc<dyn BatchedEscalationReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedEscalationReservationStore> {
    Rc::new(CachedBatchedEscalationReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedEscalationReservation>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched escalation reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationReservationQuery {
    pub region: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedEscalationReservationQuery {
    pub fn matches(&self, item: &BatchedEscalationReservation) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[BatchedEscalationReservation]) -> Vec<BatchedEscalationReservation> {
        let matched = collections::filter(items, |candidate: &BatchedEscalationReservation| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_escalation_reservations(
    store: Rc<dyn BatchedEscalationReservationStore>,
    query: &BatchedEscalationReservationQuery,
) -> Vec<BatchedEscalationReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedEscalationReservationStore> = new_memory_batched_escalation_reservation_store();
        let item = new_batched_escalation_reservation("code-924", "name-938");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
