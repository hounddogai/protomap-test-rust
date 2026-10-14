use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped timeline reservations by code.
pub trait CappedTimelineReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedTimelineReservation>;
    fn save(&self, item: &CappedTimelineReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedTimelineReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedTimelineReservationStore {
    pub items: RefCell<BTreeMap<String, CappedTimelineReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedTimelineReservationStore for MemoryCappedTimelineReservationStore {
    fn find(&self, code: &str) -> Option<CappedTimelineReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedTimelineReservation) {
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

    fn list(&self) -> Vec<CappedTimelineReservation> {
        let mut out: Vec<CappedTimelineReservation> = Vec::new();
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

pub fn new_memory_capped_timeline_reservation_store() -> Rc<dyn CappedTimelineReservationStore> {
    Rc::new(MemoryCappedTimelineReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedTimelineReservationStore {
    pub inner: Rc<dyn CappedTimelineReservationStore>,
    pub cache: RefCell<cache::TtlCache<CappedTimelineReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedTimelineReservationStore for CachedCappedTimelineReservationStore {
    fn find(&self, code: &str) -> Option<CappedTimelineReservation> {
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

    fn save(&self, item: &CappedTimelineReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedTimelineReservation> {
        self.inner.list()
    }
}

pub fn new_cached_capped_timeline_reservation_store(
    inner: Rc<dyn CappedTimelineReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedTimelineReservationStore> {
    Rc::new(CachedCappedTimelineReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedTimelineReservation>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped timeline reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservationQuery {
    pub title: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedTimelineReservationQuery {
    pub fn matches(&self, item: &CappedTimelineReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedTimelineReservation]) -> Vec<CappedTimelineReservation> {
        let matched = collections::filter(items, |candidate: &CappedTimelineReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_timeline_reservations(
    store: Rc<dyn CappedTimelineReservationStore>,
    query: &CappedTimelineReservationQuery,
) -> Vec<CappedTimelineReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedTimelineReservationStore> = new_memory_capped_timeline_reservation_store();
        let item = new_capped_timeline_reservation("code-632", "title-840");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
