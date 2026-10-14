use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted credential reservations by code.
pub trait WeightedCredentialReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedCredentialReservation>;
    fn save(&self, item: &WeightedCredentialReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedCredentialReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedCredentialReservationStore {
    pub items: RefCell<BTreeMap<String, WeightedCredentialReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedCredentialReservationStore for MemoryWeightedCredentialReservationStore {
    fn find(&self, code: &str) -> Option<WeightedCredentialReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedCredentialReservation) {
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

    fn list(&self) -> Vec<WeightedCredentialReservation> {
        let mut out: Vec<WeightedCredentialReservation> = Vec::new();
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

pub fn new_memory_weighted_credential_reservation_store() -> Rc<dyn WeightedCredentialReservationStore> {
    Rc::new(MemoryWeightedCredentialReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedCredentialReservationStore {
    pub inner: Rc<dyn WeightedCredentialReservationStore>,
    pub cache: RefCell<cache::TtlCache<WeightedCredentialReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedCredentialReservationStore for CachedWeightedCredentialReservationStore {
    fn find(&self, code: &str) -> Option<WeightedCredentialReservation> {
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

    fn save(&self, item: &WeightedCredentialReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedCredentialReservation> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_credential_reservation_store(
    inner: Rc<dyn WeightedCredentialReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedCredentialReservationStore> {
    Rc::new(CachedWeightedCredentialReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedCredentialReservation>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted credential reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialReservationQuery {
    pub region: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedCredentialReservationQuery {
    pub fn matches(&self, item: &WeightedCredentialReservation) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedCredentialReservation]) -> Vec<WeightedCredentialReservation> {
        let matched = collections::filter(items, |candidate: &WeightedCredentialReservation| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_credential_reservations(
    store: Rc<dyn WeightedCredentialReservationStore>,
    query: &WeightedCredentialReservationQuery,
) -> Vec<WeightedCredentialReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedCredentialReservationStore> = new_memory_weighted_credential_reservation_store();
        let item = new_weighted_credential_reservation("code-644", "title-516");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
