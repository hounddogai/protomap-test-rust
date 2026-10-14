use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict query reservations by code.
pub trait StrictQueryReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictQueryReservation>;
    fn save(&self, item: &StrictQueryReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictQueryReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictQueryReservationStore {
    pub items: RefCell<BTreeMap<String, StrictQueryReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictQueryReservationStore for MemoryStrictQueryReservationStore {
    fn find(&self, code: &str) -> Option<StrictQueryReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictQueryReservation) {
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

    fn list(&self) -> Vec<StrictQueryReservation> {
        let mut out: Vec<StrictQueryReservation> = Vec::new();
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

pub fn new_memory_strict_query_reservation_store() -> Rc<dyn StrictQueryReservationStore> {
    Rc::new(MemoryStrictQueryReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictQueryReservationStore {
    pub inner: Rc<dyn StrictQueryReservationStore>,
    pub cache: RefCell<cache::TtlCache<StrictQueryReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictQueryReservationStore for CachedStrictQueryReservationStore {
    fn find(&self, code: &str) -> Option<StrictQueryReservation> {
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

    fn save(&self, item: &StrictQueryReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictQueryReservation> {
        self.inner.list()
    }
}

pub fn new_cached_strict_query_reservation_store(
    inner: Rc<dyn StrictQueryReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictQueryReservationStore> {
    Rc::new(CachedStrictQueryReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictQueryReservation>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict query reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryReservationQuery {
    pub label: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictQueryReservationQuery {
    pub fn matches(&self, item: &StrictQueryReservation) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictQueryReservation]) -> Vec<StrictQueryReservation> {
        let matched = collections::filter(items, |candidate: &StrictQueryReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_query_reservations(
    store: Rc<dyn StrictQueryReservationStore>,
    query: &StrictQueryReservationQuery,
) -> Vec<StrictQueryReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictQueryReservationStore> = new_memory_strict_query_reservation_store();
        let item = new_strict_query_reservation("code-390", "label-414");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
