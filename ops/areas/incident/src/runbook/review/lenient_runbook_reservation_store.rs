use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient runbook reservations by code.
pub trait LenientRunbookReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientRunbookReservation>;
    fn save(&self, item: &LenientRunbookReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientRunbookReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientRunbookReservationStore {
    pub items: RefCell<BTreeMap<String, LenientRunbookReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientRunbookReservationStore for MemoryLenientRunbookReservationStore {
    fn find(&self, code: &str) -> Option<LenientRunbookReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientRunbookReservation) {
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

    fn list(&self) -> Vec<LenientRunbookReservation> {
        let mut out: Vec<LenientRunbookReservation> = Vec::new();
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

pub fn new_memory_lenient_runbook_reservation_store() -> Rc<dyn LenientRunbookReservationStore> {
    Rc::new(MemoryLenientRunbookReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientRunbookReservationStore {
    pub inner: Rc<dyn LenientRunbookReservationStore>,
    pub cache: RefCell<cache::TtlCache<LenientRunbookReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientRunbookReservationStore for CachedLenientRunbookReservationStore {
    fn find(&self, code: &str) -> Option<LenientRunbookReservation> {
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

    fn save(&self, item: &LenientRunbookReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientRunbookReservation> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_runbook_reservation_store(
    inner: Rc<dyn LenientRunbookReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientRunbookReservationStore> {
    Rc::new(CachedLenientRunbookReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientRunbookReservation>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient runbook reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookReservationQuery {
    pub title: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientRunbookReservationQuery {
    pub fn matches(&self, item: &LenientRunbookReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[LenientRunbookReservation]) -> Vec<LenientRunbookReservation> {
        let matched = collections::filter(items, |candidate: &LenientRunbookReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_runbook_reservations(
    store: Rc<dyn LenientRunbookReservationStore>,
    query: &LenientRunbookReservationQuery,
) -> Vec<LenientRunbookReservation> {
    query.apply(&store.list())
}
