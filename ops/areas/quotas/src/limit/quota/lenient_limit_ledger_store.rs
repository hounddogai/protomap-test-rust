use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient limit ledgers by code.
pub trait LenientLimitLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientLimitLedger>;
    fn save(&self, item: &LenientLimitLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientLimitLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientLimitLedgerStore {
    pub items: RefCell<BTreeMap<String, LenientLimitLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientLimitLedgerStore for MemoryLenientLimitLedgerStore {
    fn find(&self, code: &str) -> Option<LenientLimitLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientLimitLedger) {
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

    fn list(&self) -> Vec<LenientLimitLedger> {
        let mut out: Vec<LenientLimitLedger> = Vec::new();
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

pub fn new_memory_lenient_limit_ledger_store() -> Rc<dyn LenientLimitLedgerStore> {
    Rc::new(MemoryLenientLimitLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientLimitLedgerStore {
    pub inner: Rc<dyn LenientLimitLedgerStore>,
    pub cache: RefCell<cache::TtlCache<LenientLimitLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientLimitLedgerStore for CachedLenientLimitLedgerStore {
    fn find(&self, code: &str) -> Option<LenientLimitLedger> {
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

    fn save(&self, item: &LenientLimitLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientLimitLedger> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_limit_ledger_store(
    inner: Rc<dyn LenientLimitLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientLimitLedgerStore> {
    Rc::new(CachedLenientLimitLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientLimitLedger>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient limit ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitLedgerQuery {
    pub name: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientLimitLedgerQuery {
    pub fn matches(&self, item: &LenientLimitLedger) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientLimitLedger]) -> Vec<LenientLimitLedger> {
        let matched = collections::filter(items, |candidate: &LenientLimitLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_limit_ledgers(
    store: Rc<dyn LenientLimitLedgerStore>,
    query: &LenientLimitLedgerQuery,
) -> Vec<LenientLimitLedger> {
    query.apply(&store.list())
}
