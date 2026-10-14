use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default reservation assignments by code.
pub trait DefaultReservationAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultReservationAssignment>;
    fn save(&self, item: &DefaultReservationAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultReservationAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultReservationAssignmentStore {
    pub items: RefCell<BTreeMap<String, DefaultReservationAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultReservationAssignmentStore for MemoryDefaultReservationAssignmentStore {
    fn find(&self, code: &str) -> Option<DefaultReservationAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultReservationAssignment) {
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

    fn list(&self) -> Vec<DefaultReservationAssignment> {
        let mut out: Vec<DefaultReservationAssignment> = Vec::new();
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

pub fn new_memory_default_reservation_assignment_store() -> Rc<dyn DefaultReservationAssignmentStore> {
    Rc::new(MemoryDefaultReservationAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultReservationAssignmentStore {
    pub inner: Rc<dyn DefaultReservationAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<DefaultReservationAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultReservationAssignmentStore for CachedDefaultReservationAssignmentStore {
    fn find(&self, code: &str) -> Option<DefaultReservationAssignment> {
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

    fn save(&self, item: &DefaultReservationAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultReservationAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_default_reservation_assignment_store(
    inner: Rc<dyn DefaultReservationAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultReservationAssignmentStore> {
    Rc::new(CachedDefaultReservationAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultReservationAssignment>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default reservation assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignmentQuery {
    pub name: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultReservationAssignmentQuery {
    pub fn matches(&self, item: &DefaultReservationAssignment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultReservationAssignment]) -> Vec<DefaultReservationAssignment> {
        let matched = collections::filter(items, |candidate: &DefaultReservationAssignment| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_reservation_assignments(
    store: Rc<dyn DefaultReservationAssignmentStore>,
    query: &DefaultReservationAssignmentQuery,
) -> Vec<DefaultReservationAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultReservationAssignmentStore> = new_memory_default_reservation_assignment_store();
        let item = new_default_reservation_assignment("code-321", "name-693");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
