use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy identity ledgers by code.
pub trait LegacyIdentityLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyIdentityLedger>;
    fn save(&self, item: &LegacyIdentityLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyIdentityLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyIdentityLedgerStore {
    pub items: RefCell<BTreeMap<String, LegacyIdentityLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyIdentityLedgerStore for MemoryLegacyIdentityLedgerStore {
    fn find(&self, code: &str) -> Option<LegacyIdentityLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyIdentityLedger) {
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

    fn list(&self) -> Vec<LegacyIdentityLedger> {
        let mut out: Vec<LegacyIdentityLedger> = Vec::new();
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

pub fn new_memory_legacy_identity_ledger_store() -> Rc<dyn LegacyIdentityLedgerStore> {
    Rc::new(MemoryLegacyIdentityLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyIdentityLedgerStore {
    pub inner: Rc<dyn LegacyIdentityLedgerStore>,
    pub cache: RefCell<cache::TtlCache<LegacyIdentityLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyIdentityLedgerStore for CachedLegacyIdentityLedgerStore {
    fn find(&self, code: &str) -> Option<LegacyIdentityLedger> {
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

    fn save(&self, item: &LegacyIdentityLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyIdentityLedger> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_identity_ledger_store(
    inner: Rc<dyn LegacyIdentityLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyIdentityLedgerStore> {
    Rc::new(CachedLegacyIdentityLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyIdentityLedger>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy identity ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIdentityLedgerQuery {
    pub channel: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyIdentityLedgerQuery {
    pub fn matches(&self, item: &LegacyIdentityLedger) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyIdentityLedger]) -> Vec<LegacyIdentityLedger> {
        let matched = collections::filter(items, |candidate: &LegacyIdentityLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_identity_ledgers(
    store: Rc<dyn LegacyIdentityLedgerStore>,
    query: &LegacyIdentityLedgerQuery,
) -> Vec<LegacyIdentityLedger> {
    query.apply(&store.list())
}
