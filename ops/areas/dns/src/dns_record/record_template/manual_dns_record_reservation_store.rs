use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual dns record reservations by code.
pub trait ManualDnsRecordReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualDnsRecordReservation>;
    fn save(&self, item: &ManualDnsRecordReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualDnsRecordReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualDnsRecordReservationStore {
    pub items: RefCell<BTreeMap<String, ManualDnsRecordReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualDnsRecordReservationStore for MemoryManualDnsRecordReservationStore {
    fn find(&self, code: &str) -> Option<ManualDnsRecordReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualDnsRecordReservation) {
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

    fn list(&self) -> Vec<ManualDnsRecordReservation> {
        let mut out: Vec<ManualDnsRecordReservation> = Vec::new();
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

pub fn new_memory_manual_dns_record_reservation_store() -> Rc<dyn ManualDnsRecordReservationStore> {
    Rc::new(MemoryManualDnsRecordReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualDnsRecordReservationStore {
    pub inner: Rc<dyn ManualDnsRecordReservationStore>,
    pub cache: RefCell<cache::TtlCache<ManualDnsRecordReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualDnsRecordReservationStore for CachedManualDnsRecordReservationStore {
    fn find(&self, code: &str) -> Option<ManualDnsRecordReservation> {
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

    fn save(&self, item: &ManualDnsRecordReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualDnsRecordReservation> {
        self.inner.list()
    }
}

pub fn new_cached_manual_dns_record_reservation_store(
    inner: Rc<dyn ManualDnsRecordReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualDnsRecordReservationStore> {
    Rc::new(CachedManualDnsRecordReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualDnsRecordReservation>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual dns record reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordReservationQuery {
    pub title: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualDnsRecordReservationQuery {
    pub fn matches(&self, item: &ManualDnsRecordReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[ManualDnsRecordReservation]) -> Vec<ManualDnsRecordReservation> {
        let matched = collections::filter(items, |candidate: &ManualDnsRecordReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_dns_record_reservations(
    store: Rc<dyn ManualDnsRecordReservationStore>,
    query: &ManualDnsRecordReservationQuery,
) -> Vec<ManualDnsRecordReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualDnsRecordReservationStore> = new_memory_manual_dns_record_reservation_store();
        let item = new_manual_dns_record_reservation("code-683", "title-964");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
