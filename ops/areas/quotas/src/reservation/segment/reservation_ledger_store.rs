use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps reservation ledgers by code.
pub trait ReservationLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReservationLedger>;
    fn save(&self, item: &ReservationLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReservationLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservationLedgerStore {
    pub items: RefCell<BTreeMap<String, ReservationLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl ReservationLedgerStore for MemoryReservationLedgerStore {
    fn find(&self, code: &str) -> Option<ReservationLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReservationLedger) {
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

    fn list(&self) -> Vec<ReservationLedger> {
        let mut out: Vec<ReservationLedger> = Vec::new();
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

pub fn new_memory_reservation_ledger_store() -> Rc<dyn ReservationLedgerStore> {
    Rc::new(MemoryReservationLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReservationLedgerStore {
    pub inner: Rc<dyn ReservationLedgerStore>,
    pub cache: RefCell<cache::TtlCache<ReservationLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReservationLedgerStore for CachedReservationLedgerStore {
    fn find(&self, code: &str) -> Option<ReservationLedger> {
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

    fn save(&self, item: &ReservationLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReservationLedger> {
        self.inner.list()
    }
}

pub fn new_cached_reservation_ledger_store(
    inner: Rc<dyn ReservationLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReservationLedgerStore> {
    Rc::new(CachedReservationLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReservationLedger>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects reservation ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationLedgerQuery {
    pub region: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReservationLedgerQuery {
    pub fn matches(&self, item: &ReservationLedger) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReservationLedger]) -> Vec<ReservationLedger> {
        let matched = collections::filter(items, |candidate: &ReservationLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_reservation_ledgers(
    store: Rc<dyn ReservationLedgerStore>,
    query: &ReservationLedgerQuery,
) -> Vec<ReservationLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReservationLedgerStore> = new_memory_reservation_ledger_store();
        let item = new_reservation_ledger("code-423", "name-829");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
