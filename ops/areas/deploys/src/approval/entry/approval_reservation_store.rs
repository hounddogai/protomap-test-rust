use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval reservations by code.
pub trait ApprovalReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalReservation>;
    fn save(&self, item: &ApprovalReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalReservationStore {
    pub items: RefCell<BTreeMap<String, ApprovalReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalReservationStore for MemoryApprovalReservationStore {
    fn find(&self, code: &str) -> Option<ApprovalReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalReservation) {
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

    fn list(&self) -> Vec<ApprovalReservation> {
        let mut out: Vec<ApprovalReservation> = Vec::new();
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

pub fn new_memory_approval_reservation_store() -> Rc<dyn ApprovalReservationStore> {
    Rc::new(MemoryApprovalReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalReservationStore {
    pub inner: Rc<dyn ApprovalReservationStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalReservationStore for CachedApprovalReservationStore {
    fn find(&self, code: &str) -> Option<ApprovalReservation> {
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

    fn save(&self, item: &ApprovalReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalReservation> {
        self.inner.list()
    }
}

pub fn new_cached_approval_reservation_store(
    inner: Rc<dyn ApprovalReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalReservationStore> {
    Rc::new(CachedApprovalReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalReservation>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalReservationQuery {
    pub title: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalReservationQuery {
    pub fn matches(&self, item: &ApprovalReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ApprovalReservation]) -> Vec<ApprovalReservation> {
        let matched = collections::filter(items, |candidate: &ApprovalReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_reservations(
    store: Rc<dyn ApprovalReservationStore>,
    query: &ApprovalReservationQuery,
) -> Vec<ApprovalReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ApprovalReservationStore> = new_memory_approval_reservation_store();
        let item = new_approval_reservation("code-294", "title-577");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
