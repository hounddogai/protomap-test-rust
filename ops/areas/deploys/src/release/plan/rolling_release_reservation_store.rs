use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling release reservations by code.
pub trait RollingReleaseReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingReleaseReservation>;
    fn save(&self, item: &RollingReleaseReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingReleaseReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingReleaseReservationStore {
    pub items: RefCell<BTreeMap<String, RollingReleaseReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingReleaseReservationStore for MemoryRollingReleaseReservationStore {
    fn find(&self, code: &str) -> Option<RollingReleaseReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingReleaseReservation) {
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

    fn list(&self) -> Vec<RollingReleaseReservation> {
        let mut out: Vec<RollingReleaseReservation> = Vec::new();
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

pub fn new_memory_rolling_release_reservation_store() -> Rc<dyn RollingReleaseReservationStore> {
    Rc::new(MemoryRollingReleaseReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingReleaseReservationStore {
    pub inner: Rc<dyn RollingReleaseReservationStore>,
    pub cache: RefCell<cache::TtlCache<RollingReleaseReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingReleaseReservationStore for CachedRollingReleaseReservationStore {
    fn find(&self, code: &str) -> Option<RollingReleaseReservation> {
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

    fn save(&self, item: &RollingReleaseReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingReleaseReservation> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_release_reservation_store(
    inner: Rc<dyn RollingReleaseReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingReleaseReservationStore> {
    Rc::new(CachedRollingReleaseReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingReleaseReservation>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling release reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseReservationQuery {
    pub region: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingReleaseReservationQuery {
    pub fn matches(&self, item: &RollingReleaseReservation) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingReleaseReservation]) -> Vec<RollingReleaseReservation> {
        let matched = collections::filter(items, |candidate: &RollingReleaseReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_release_reservations(
    store: Rc<dyn RollingReleaseReservationStore>,
    query: &RollingReleaseReservationQuery,
) -> Vec<RollingReleaseReservation> {
    query.apply(&store.list())
}
