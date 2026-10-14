use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive nonce reservations by code.
pub trait AdaptiveNonceReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveNonceReservation>;
    fn save(&self, item: &AdaptiveNonceReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveNonceReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveNonceReservationStore {
    pub items: RefCell<BTreeMap<String, AdaptiveNonceReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveNonceReservationStore for MemoryAdaptiveNonceReservationStore {
    fn find(&self, code: &str) -> Option<AdaptiveNonceReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveNonceReservation) {
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

    fn list(&self) -> Vec<AdaptiveNonceReservation> {
        let mut out: Vec<AdaptiveNonceReservation> = Vec::new();
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

pub fn new_memory_adaptive_nonce_reservation_store() -> Rc<dyn AdaptiveNonceReservationStore> {
    Rc::new(MemoryAdaptiveNonceReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveNonceReservationStore {
    pub inner: Rc<dyn AdaptiveNonceReservationStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveNonceReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveNonceReservationStore for CachedAdaptiveNonceReservationStore {
    fn find(&self, code: &str) -> Option<AdaptiveNonceReservation> {
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

    fn save(&self, item: &AdaptiveNonceReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveNonceReservation> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_nonce_reservation_store(
    inner: Rc<dyn AdaptiveNonceReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveNonceReservationStore> {
    Rc::new(CachedAdaptiveNonceReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveNonceReservation>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive nonce reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceReservationQuery {
    pub namespace: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveNonceReservationQuery {
    pub fn matches(&self, item: &AdaptiveNonceReservation) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[AdaptiveNonceReservation]) -> Vec<AdaptiveNonceReservation> {
        let matched = collections::filter(items, |candidate: &AdaptiveNonceReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_nonce_reservations(
    store: Rc<dyn AdaptiveNonceReservationStore>,
    query: &AdaptiveNonceReservationQuery,
) -> Vec<AdaptiveNonceReservation> {
    query.apply(&store.list())
}
