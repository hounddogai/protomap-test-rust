use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default grant ledgers by code.
pub trait DefaultGrantLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultGrantLedger>;
    fn save(&self, item: &DefaultGrantLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultGrantLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultGrantLedgerStore {
    pub items: RefCell<BTreeMap<String, DefaultGrantLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultGrantLedgerStore for MemoryDefaultGrantLedgerStore {
    fn find(&self, code: &str) -> Option<DefaultGrantLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultGrantLedger) {
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

    fn list(&self) -> Vec<DefaultGrantLedger> {
        let mut out: Vec<DefaultGrantLedger> = Vec::new();
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

pub fn new_memory_default_grant_ledger_store() -> Rc<dyn DefaultGrantLedgerStore> {
    Rc::new(MemoryDefaultGrantLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultGrantLedgerStore {
    pub inner: Rc<dyn DefaultGrantLedgerStore>,
    pub cache: RefCell<cache::TtlCache<DefaultGrantLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultGrantLedgerStore for CachedDefaultGrantLedgerStore {
    fn find(&self, code: &str) -> Option<DefaultGrantLedger> {
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

    fn save(&self, item: &DefaultGrantLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultGrantLedger> {
        self.inner.list()
    }
}

pub fn new_cached_default_grant_ledger_store(
    inner: Rc<dyn DefaultGrantLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultGrantLedgerStore> {
    Rc::new(CachedDefaultGrantLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultGrantLedger>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default grant ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantLedgerQuery {
    pub cluster: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultGrantLedgerQuery {
    pub fn matches(&self, item: &DefaultGrantLedger) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[DefaultGrantLedger]) -> Vec<DefaultGrantLedger> {
        let matched = collections::filter(items, |candidate: &DefaultGrantLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_grant_ledgers(
    store: Rc<dyn DefaultGrantLedgerStore>,
    query: &DefaultGrantLedgerQuery,
) -> Vec<DefaultGrantLedger> {
    query.apply(&store.list())
}
