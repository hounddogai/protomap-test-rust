use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query ledgers by code.
pub trait QueryLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryLedger>;
    fn save(&self, item: &QueryLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryLedgerStore {
    pub items: RefCell<BTreeMap<String, QueryLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryLedgerStore for MemoryQueryLedgerStore {
    fn find(&self, code: &str) -> Option<QueryLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryLedger) {
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

    fn list(&self) -> Vec<QueryLedger> {
        let mut out: Vec<QueryLedger> = Vec::new();
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

pub fn new_memory_query_ledger_store() -> Rc<dyn QueryLedgerStore> {
    Rc::new(MemoryQueryLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryLedgerStore {
    pub inner: Rc<dyn QueryLedgerStore>,
    pub cache: RefCell<cache::TtlCache<QueryLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryLedgerStore for CachedQueryLedgerStore {
    fn find(&self, code: &str) -> Option<QueryLedger> {
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

    fn save(&self, item: &QueryLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryLedger> {
        self.inner.list()
    }
}

pub fn new_cached_query_ledger_store(
    inner: Rc<dyn QueryLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryLedgerStore> {
    Rc::new(CachedQueryLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryLedger>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryLedgerQuery {
    pub unit: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryLedgerQuery {
    pub fn matches(&self, item: &QueryLedger) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QueryLedger]) -> Vec<QueryLedger> {
        let matched = collections::filter(items, |candidate: &QueryLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_ledgers(store: Rc<dyn QueryLedgerStore>, query: &QueryLedgerQuery) -> Vec<QueryLedger> {
    query.apply(&store.list())
}
