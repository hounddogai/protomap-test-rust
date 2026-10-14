use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled verifier ledgers by code.
pub trait ScheduledVerifierLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledVerifierLedger>;
    fn save(&self, item: &ScheduledVerifierLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledVerifierLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledVerifierLedgerStore {
    pub items: RefCell<BTreeMap<String, ScheduledVerifierLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledVerifierLedgerStore for MemoryScheduledVerifierLedgerStore {
    fn find(&self, code: &str) -> Option<ScheduledVerifierLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledVerifierLedger) {
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

    fn list(&self) -> Vec<ScheduledVerifierLedger> {
        let mut out: Vec<ScheduledVerifierLedger> = Vec::new();
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

pub fn new_memory_scheduled_verifier_ledger_store() -> Rc<dyn ScheduledVerifierLedgerStore> {
    Rc::new(MemoryScheduledVerifierLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledVerifierLedgerStore {
    pub inner: Rc<dyn ScheduledVerifierLedgerStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledVerifierLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledVerifierLedgerStore for CachedScheduledVerifierLedgerStore {
    fn find(&self, code: &str) -> Option<ScheduledVerifierLedger> {
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

    fn save(&self, item: &ScheduledVerifierLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledVerifierLedger> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_verifier_ledger_store(
    inner: Rc<dyn ScheduledVerifierLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledVerifierLedgerStore> {
    Rc::new(CachedScheduledVerifierLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledVerifierLedger>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled verifier ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierLedgerQuery {
    pub country: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledVerifierLedgerQuery {
    pub fn matches(&self, item: &ScheduledVerifierLedger) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[ScheduledVerifierLedger]) -> Vec<ScheduledVerifierLedger> {
        let matched = collections::filter(items, |candidate: &ScheduledVerifierLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_verifier_ledgers(
    store: Rc<dyn ScheduledVerifierLedgerStore>,
    query: &ScheduledVerifierLedgerQuery,
) -> Vec<ScheduledVerifierLedger> {
    query.apply(&store.list())
}
