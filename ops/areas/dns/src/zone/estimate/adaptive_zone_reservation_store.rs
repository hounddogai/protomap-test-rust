use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive zone reservations by code.
pub trait AdaptiveZoneReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveZoneReservation>;
    fn save(&self, item: &AdaptiveZoneReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveZoneReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveZoneReservationStore {
    pub items: RefCell<BTreeMap<String, AdaptiveZoneReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveZoneReservationStore for MemoryAdaptiveZoneReservationStore {
    fn find(&self, code: &str) -> Option<AdaptiveZoneReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveZoneReservation) {
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

    fn list(&self) -> Vec<AdaptiveZoneReservation> {
        let mut out: Vec<AdaptiveZoneReservation> = Vec::new();
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

pub fn new_memory_adaptive_zone_reservation_store() -> Rc<dyn AdaptiveZoneReservationStore> {
    Rc::new(MemoryAdaptiveZoneReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveZoneReservationStore {
    pub inner: Rc<dyn AdaptiveZoneReservationStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveZoneReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveZoneReservationStore for CachedAdaptiveZoneReservationStore {
    fn find(&self, code: &str) -> Option<AdaptiveZoneReservation> {
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

    fn save(&self, item: &AdaptiveZoneReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveZoneReservation> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_zone_reservation_store(
    inner: Rc<dyn AdaptiveZoneReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveZoneReservationStore> {
    Rc::new(CachedAdaptiveZoneReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveZoneReservation>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive zone reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveZoneReservationQuery {
    pub source: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveZoneReservationQuery {
    pub fn matches(&self, item: &AdaptiveZoneReservation) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveZoneReservation]) -> Vec<AdaptiveZoneReservation> {
        let matched = collections::filter(items, |candidate: &AdaptiveZoneReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_zone_reservations(
    store: Rc<dyn AdaptiveZoneReservationStore>,
    query: &AdaptiveZoneReservationQuery,
) -> Vec<AdaptiveZoneReservation> {
    query.apply(&store.list())
}
