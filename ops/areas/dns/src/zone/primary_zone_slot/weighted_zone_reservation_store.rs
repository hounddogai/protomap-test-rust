use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted zone reservations by code.
pub trait WeightedZoneReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedZoneReservation>;
    fn save(&self, item: &WeightedZoneReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedZoneReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedZoneReservationStore {
    pub items: RefCell<BTreeMap<String, WeightedZoneReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedZoneReservationStore for MemoryWeightedZoneReservationStore {
    fn find(&self, code: &str) -> Option<WeightedZoneReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedZoneReservation) {
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

    fn list(&self) -> Vec<WeightedZoneReservation> {
        let mut out: Vec<WeightedZoneReservation> = Vec::new();
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

pub fn new_memory_weighted_zone_reservation_store() -> Rc<dyn WeightedZoneReservationStore> {
    Rc::new(MemoryWeightedZoneReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedZoneReservationStore {
    pub inner: Rc<dyn WeightedZoneReservationStore>,
    pub cache: RefCell<cache::TtlCache<WeightedZoneReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedZoneReservationStore for CachedWeightedZoneReservationStore {
    fn find(&self, code: &str) -> Option<WeightedZoneReservation> {
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

    fn save(&self, item: &WeightedZoneReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedZoneReservation> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_zone_reservation_store(
    inner: Rc<dyn WeightedZoneReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedZoneReservationStore> {
    Rc::new(CachedWeightedZoneReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedZoneReservation>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted zone reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneReservationQuery {
    pub title: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedZoneReservationQuery {
    pub fn matches(&self, item: &WeightedZoneReservation) -> bool {
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

    pub fn apply(&self, items: &[WeightedZoneReservation]) -> Vec<WeightedZoneReservation> {
        let matched = collections::filter(items, |candidate: &WeightedZoneReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_zone_reservations(
    store: Rc<dyn WeightedZoneReservationStore>,
    query: &WeightedZoneReservationQuery,
) -> Vec<WeightedZoneReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedZoneReservationStore> = new_memory_weighted_zone_reservation_store();
        let item = new_weighted_zone_reservation("code-622", "title-401");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
