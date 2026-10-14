use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient segment reservations by code.
pub trait LenientSegmentReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientSegmentReservation>;
    fn save(&self, item: &LenientSegmentReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientSegmentReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientSegmentReservationStore {
    pub items: RefCell<BTreeMap<String, LenientSegmentReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientSegmentReservationStore for MemoryLenientSegmentReservationStore {
    fn find(&self, code: &str) -> Option<LenientSegmentReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientSegmentReservation) {
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

    fn list(&self) -> Vec<LenientSegmentReservation> {
        let mut out: Vec<LenientSegmentReservation> = Vec::new();
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

pub fn new_memory_lenient_segment_reservation_store() -> Rc<dyn LenientSegmentReservationStore> {
    Rc::new(MemoryLenientSegmentReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientSegmentReservationStore {
    pub inner: Rc<dyn LenientSegmentReservationStore>,
    pub cache: RefCell<cache::TtlCache<LenientSegmentReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientSegmentReservationStore for CachedLenientSegmentReservationStore {
    fn find(&self, code: &str) -> Option<LenientSegmentReservation> {
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

    fn save(&self, item: &LenientSegmentReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientSegmentReservation> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_segment_reservation_store(
    inner: Rc<dyn LenientSegmentReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientSegmentReservationStore> {
    Rc::new(CachedLenientSegmentReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientSegmentReservation>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient segment reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentReservationQuery {
    pub namespace: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientSegmentReservationQuery {
    pub fn matches(&self, item: &LenientSegmentReservation) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientSegmentReservation]) -> Vec<LenientSegmentReservation> {
        let matched = collections::filter(items, |candidate: &LenientSegmentReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_segment_reservations(
    store: Rc<dyn LenientSegmentReservationStore>,
    query: &LenientSegmentReservationQuery,
) -> Vec<LenientSegmentReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientSegmentReservationStore> = new_memory_lenient_segment_reservation_store();
        let item = new_lenient_segment_reservation("code-496", "name-999");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
