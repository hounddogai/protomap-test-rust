use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict runbook reservations by code.
pub trait StrictRunbookReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictRunbookReservation>;
    fn save(&self, item: &StrictRunbookReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictRunbookReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictRunbookReservationStore {
    pub items: RefCell<BTreeMap<String, StrictRunbookReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictRunbookReservationStore for MemoryStrictRunbookReservationStore {
    fn find(&self, code: &str) -> Option<StrictRunbookReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictRunbookReservation) {
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

    fn list(&self) -> Vec<StrictRunbookReservation> {
        let mut out: Vec<StrictRunbookReservation> = Vec::new();
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

pub fn new_memory_strict_runbook_reservation_store() -> Rc<dyn StrictRunbookReservationStore> {
    Rc::new(MemoryStrictRunbookReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictRunbookReservationStore {
    pub inner: Rc<dyn StrictRunbookReservationStore>,
    pub cache: RefCell<cache::TtlCache<StrictRunbookReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictRunbookReservationStore for CachedStrictRunbookReservationStore {
    fn find(&self, code: &str) -> Option<StrictRunbookReservation> {
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

    fn save(&self, item: &StrictRunbookReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictRunbookReservation> {
        self.inner.list()
    }
}

pub fn new_cached_strict_runbook_reservation_store(
    inner: Rc<dyn StrictRunbookReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictRunbookReservationStore> {
    Rc::new(CachedStrictRunbookReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictRunbookReservation>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict runbook reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookReservationQuery {
    pub scope: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictRunbookReservationQuery {
    pub fn matches(&self, item: &StrictRunbookReservation) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[StrictRunbookReservation]) -> Vec<StrictRunbookReservation> {
        let matched = collections::filter(items, |candidate: &StrictRunbookReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_runbook_reservations(
    store: Rc<dyn StrictRunbookReservationStore>,
    query: &StrictRunbookReservationQuery,
) -> Vec<StrictRunbookReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictRunbookReservationStore> = new_memory_strict_runbook_reservation_store();
        let item = new_strict_runbook_reservation("code-991", "name-394");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
