use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps silence reservations by code.
pub trait SilenceReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SilenceReservation>;
    fn save(&self, item: &SilenceReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SilenceReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySilenceReservationStore {
    pub items: RefCell<BTreeMap<String, SilenceReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl SilenceReservationStore for MemorySilenceReservationStore {
    fn find(&self, code: &str) -> Option<SilenceReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SilenceReservation) {
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

    fn list(&self) -> Vec<SilenceReservation> {
        let mut out: Vec<SilenceReservation> = Vec::new();
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

pub fn new_memory_silence_reservation_store() -> Rc<dyn SilenceReservationStore> {
    Rc::new(MemorySilenceReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSilenceReservationStore {
    pub inner: Rc<dyn SilenceReservationStore>,
    pub cache: RefCell<cache::TtlCache<SilenceReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SilenceReservationStore for CachedSilenceReservationStore {
    fn find(&self, code: &str) -> Option<SilenceReservation> {
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

    fn save(&self, item: &SilenceReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SilenceReservation> {
        self.inner.list()
    }
}

pub fn new_cached_silence_reservation_store(
    inner: Rc<dyn SilenceReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SilenceReservationStore> {
    Rc::new(CachedSilenceReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SilenceReservation>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects silence reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReservationQuery {
    pub name: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SilenceReservationQuery {
    pub fn matches(&self, item: &SilenceReservation) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SilenceReservation]) -> Vec<SilenceReservation> {
        let matched = collections::filter(items, |candidate: &SilenceReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_silence_reservations(
    store: Rc<dyn SilenceReservationStore>,
    query: &SilenceReservationQuery,
) -> Vec<SilenceReservation> {
    query.apply(&store.list())
}
