use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered reservation assignments by code.
pub trait TieredReservationAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredReservationAssignment>;
    fn save(&self, item: &TieredReservationAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredReservationAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredReservationAssignmentStore {
    pub items: RefCell<BTreeMap<String, TieredReservationAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredReservationAssignmentStore for MemoryTieredReservationAssignmentStore {
    fn find(&self, code: &str) -> Option<TieredReservationAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredReservationAssignment) {
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

    fn list(&self) -> Vec<TieredReservationAssignment> {
        let mut out: Vec<TieredReservationAssignment> = Vec::new();
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

pub fn new_memory_tiered_reservation_assignment_store() -> Rc<dyn TieredReservationAssignmentStore> {
    Rc::new(MemoryTieredReservationAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredReservationAssignmentStore {
    pub inner: Rc<dyn TieredReservationAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<TieredReservationAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredReservationAssignmentStore for CachedTieredReservationAssignmentStore {
    fn find(&self, code: &str) -> Option<TieredReservationAssignment> {
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

    fn save(&self, item: &TieredReservationAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredReservationAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_reservation_assignment_store(
    inner: Rc<dyn TieredReservationAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredReservationAssignmentStore> {
    Rc::new(CachedTieredReservationAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredReservationAssignment>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered reservation assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationAssignmentQuery {
    pub label: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredReservationAssignmentQuery {
    pub fn matches(&self, item: &TieredReservationAssignment) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredReservationAssignment]) -> Vec<TieredReservationAssignment> {
        let matched = collections::filter(items, |candidate: &TieredReservationAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_reservation_assignments(
    store: Rc<dyn TieredReservationAssignmentStore>,
    query: &TieredReservationAssignmentQuery,
) -> Vec<TieredReservationAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredReservationAssignmentStore> = new_memory_tiered_reservation_assignment_store();
        let item = new_tiered_reservation_assignment("code-995", "label-586");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
