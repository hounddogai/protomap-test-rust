use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps page ledgers by code.
pub trait PageLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PageLedger>;
    fn save(&self, item: &PageLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PageLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPageLedgerStore {
    pub items: RefCell<BTreeMap<String, PageLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl PageLedgerStore for MemoryPageLedgerStore {
    fn find(&self, code: &str) -> Option<PageLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PageLedger) {
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

    fn list(&self) -> Vec<PageLedger> {
        let mut out: Vec<PageLedger> = Vec::new();
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

pub fn new_memory_page_ledger_store() -> Rc<dyn PageLedgerStore> {
    Rc::new(MemoryPageLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPageLedgerStore {
    pub inner: Rc<dyn PageLedgerStore>,
    pub cache: RefCell<cache::TtlCache<PageLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PageLedgerStore for CachedPageLedgerStore {
    fn find(&self, code: &str) -> Option<PageLedger> {
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

    fn save(&self, item: &PageLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PageLedger> {
        self.inner.list()
    }
}

pub fn new_cached_page_ledger_store(
    inner: Rc<dyn PageLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PageLedgerStore> {
    Rc::new(CachedPageLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PageLedger>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects page ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PageLedgerQuery {
    pub namespace: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PageLedgerQuery {
    pub fn matches(&self, item: &PageLedger) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PageLedger]) -> Vec<PageLedger> {
        let matched = collections::filter(items, |candidate: &PageLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_page_ledgers(store: Rc<dyn PageLedgerStore>, query: &PageLedgerQuery) -> Vec<PageLedger> {
    query.apply(&store.list())
}
