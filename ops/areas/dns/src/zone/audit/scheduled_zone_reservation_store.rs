use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled zone reservations by code.
pub trait ScheduledZoneReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledZoneReservation>;
    fn save(&self, item: &ScheduledZoneReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledZoneReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledZoneReservationStore {
    pub items: RefCell<BTreeMap<String, ScheduledZoneReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledZoneReservationStore for MemoryScheduledZoneReservationStore {
    fn find(&self, code: &str) -> Option<ScheduledZoneReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledZoneReservation) {
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

    fn list(&self) -> Vec<ScheduledZoneReservation> {
        let mut out: Vec<ScheduledZoneReservation> = Vec::new();
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

pub fn new_memory_scheduled_zone_reservation_store() -> Rc<dyn ScheduledZoneReservationStore> {
    Rc::new(MemoryScheduledZoneReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledZoneReservationStore {
    pub inner: Rc<dyn ScheduledZoneReservationStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledZoneReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledZoneReservationStore for CachedScheduledZoneReservationStore {
    fn find(&self, code: &str) -> Option<ScheduledZoneReservation> {
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

    fn save(&self, item: &ScheduledZoneReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledZoneReservation> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_zone_reservation_store(
    inner: Rc<dyn ScheduledZoneReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledZoneReservationStore> {
    Rc::new(CachedScheduledZoneReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledZoneReservation>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled zone reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneReservationQuery {
    pub category: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledZoneReservationQuery {
    pub fn matches(&self, item: &ScheduledZoneReservation) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[ScheduledZoneReservation]) -> Vec<ScheduledZoneReservation> {
        let matched = collections::filter(items, |candidate: &ScheduledZoneReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_zone_reservations(
    store: Rc<dyn ScheduledZoneReservationStore>,
    query: &ScheduledZoneReservationQuery,
) -> Vec<ScheduledZoneReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledZoneReservationStore> = new_memory_scheduled_zone_reservation_store();
        let item = new_scheduled_zone_reservation("code-223", "label-474");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
