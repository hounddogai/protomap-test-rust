use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps allowlist reservations by code.
pub trait AllowlistReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AllowlistReservation>;
    fn save(&self, item: &AllowlistReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AllowlistReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAllowlistReservationStore {
    pub items: RefCell<BTreeMap<String, AllowlistReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl AllowlistReservationStore for MemoryAllowlistReservationStore {
    fn find(&self, code: &str) -> Option<AllowlistReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AllowlistReservation) {
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

    fn list(&self) -> Vec<AllowlistReservation> {
        let mut out: Vec<AllowlistReservation> = Vec::new();
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

pub fn new_memory_allowlist_reservation_store() -> Rc<dyn AllowlistReservationStore> {
    Rc::new(MemoryAllowlistReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAllowlistReservationStore {
    pub inner: Rc<dyn AllowlistReservationStore>,
    pub cache: RefCell<cache::TtlCache<AllowlistReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AllowlistReservationStore for CachedAllowlistReservationStore {
    fn find(&self, code: &str) -> Option<AllowlistReservation> {
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

    fn save(&self, item: &AllowlistReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AllowlistReservation> {
        self.inner.list()
    }
}

pub fn new_cached_allowlist_reservation_store(
    inner: Rc<dyn AllowlistReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AllowlistReservationStore> {
    Rc::new(CachedAllowlistReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AllowlistReservation>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects allowlist reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistReservationQuery {
    pub cluster: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AllowlistReservationQuery {
    pub fn matches(&self, item: &AllowlistReservation) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AllowlistReservation]) -> Vec<AllowlistReservation> {
        let matched = collections::filter(items, |candidate: &AllowlistReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_allowlist_reservations(
    store: Rc<dyn AllowlistReservationStore>,
    query: &AllowlistReservationQuery,
) -> Vec<AllowlistReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AllowlistReservationStore> = new_memory_allowlist_reservation_store();
        let item = new_allowlist_reservation("code-641", "label-838");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
