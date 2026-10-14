use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback evidence reservations by code.
pub trait FallbackEvidenceReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackEvidenceReservation>;
    fn save(&self, item: &FallbackEvidenceReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackEvidenceReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackEvidenceReservationStore {
    pub items: RefCell<BTreeMap<String, FallbackEvidenceReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackEvidenceReservationStore for MemoryFallbackEvidenceReservationStore {
    fn find(&self, code: &str) -> Option<FallbackEvidenceReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackEvidenceReservation) {
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

    fn list(&self) -> Vec<FallbackEvidenceReservation> {
        let mut out: Vec<FallbackEvidenceReservation> = Vec::new();
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

pub fn new_memory_fallback_evidence_reservation_store() -> Rc<dyn FallbackEvidenceReservationStore> {
    Rc::new(MemoryFallbackEvidenceReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackEvidenceReservationStore {
    pub inner: Rc<dyn FallbackEvidenceReservationStore>,
    pub cache: RefCell<cache::TtlCache<FallbackEvidenceReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackEvidenceReservationStore for CachedFallbackEvidenceReservationStore {
    fn find(&self, code: &str) -> Option<FallbackEvidenceReservation> {
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

    fn save(&self, item: &FallbackEvidenceReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackEvidenceReservation> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_evidence_reservation_store(
    inner: Rc<dyn FallbackEvidenceReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackEvidenceReservationStore> {
    Rc::new(CachedFallbackEvidenceReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackEvidenceReservation>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback evidence reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceReservationQuery {
    pub label: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackEvidenceReservationQuery {
    pub fn matches(&self, item: &FallbackEvidenceReservation) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackEvidenceReservation]) -> Vec<FallbackEvidenceReservation> {
        let matched = collections::filter(items, |candidate: &FallbackEvidenceReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_evidence_reservations(
    store: Rc<dyn FallbackEvidenceReservationStore>,
    query: &FallbackEvidenceReservationQuery,
) -> Vec<FallbackEvidenceReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackEvidenceReservationStore> = new_memory_fallback_evidence_reservation_store();
        let item = new_fallback_evidence_reservation("code-439", "label-185");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
