use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps ttl policy reservations by code.
pub trait TtlPolicyReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TtlPolicyReservation>;
    fn save(&self, item: &TtlPolicyReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TtlPolicyReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTtlPolicyReservationStore {
    pub items: RefCell<BTreeMap<String, TtlPolicyReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl TtlPolicyReservationStore for MemoryTtlPolicyReservationStore {
    fn find(&self, code: &str) -> Option<TtlPolicyReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TtlPolicyReservation) {
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

    fn list(&self) -> Vec<TtlPolicyReservation> {
        let mut out: Vec<TtlPolicyReservation> = Vec::new();
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

pub fn new_memory_ttl_policy_reservation_store() -> Rc<dyn TtlPolicyReservationStore> {
    Rc::new(MemoryTtlPolicyReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTtlPolicyReservationStore {
    pub inner: Rc<dyn TtlPolicyReservationStore>,
    pub cache: RefCell<cache::TtlCache<TtlPolicyReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TtlPolicyReservationStore for CachedTtlPolicyReservationStore {
    fn find(&self, code: &str) -> Option<TtlPolicyReservation> {
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

    fn save(&self, item: &TtlPolicyReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TtlPolicyReservation> {
        self.inner.list()
    }
}

pub fn new_cached_ttl_policy_reservation_store(
    inner: Rc<dyn TtlPolicyReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TtlPolicyReservationStore> {
    Rc::new(CachedTtlPolicyReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TtlPolicyReservation>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects ttl policy reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyReservationQuery {
    pub title: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TtlPolicyReservationQuery {
    pub fn matches(&self, item: &TtlPolicyReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[TtlPolicyReservation]) -> Vec<TtlPolicyReservation> {
        let matched = collections::filter(items, |candidate: &TtlPolicyReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_ttl_policy_reservations(
    store: Rc<dyn TtlPolicyReservationStore>,
    query: &TtlPolicyReservationQuery,
) -> Vec<TtlPolicyReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TtlPolicyReservationStore> = new_memory_ttl_policy_reservation_store();
        let item = new_ttl_policy_reservation("code-873", "title-622");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
