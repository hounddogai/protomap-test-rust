use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps segment ledgers by code.
pub trait SegmentLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SegmentLedger>;
    fn save(&self, item: &SegmentLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SegmentLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySegmentLedgerStore {
    pub items: RefCell<BTreeMap<String, SegmentLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl SegmentLedgerStore for MemorySegmentLedgerStore {
    fn find(&self, code: &str) -> Option<SegmentLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SegmentLedger) {
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

    fn list(&self) -> Vec<SegmentLedger> {
        let mut out: Vec<SegmentLedger> = Vec::new();
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

pub fn new_memory_segment_ledger_store() -> Rc<dyn SegmentLedgerStore> {
    Rc::new(MemorySegmentLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSegmentLedgerStore {
    pub inner: Rc<dyn SegmentLedgerStore>,
    pub cache: RefCell<cache::TtlCache<SegmentLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SegmentLedgerStore for CachedSegmentLedgerStore {
    fn find(&self, code: &str) -> Option<SegmentLedger> {
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

    fn save(&self, item: &SegmentLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SegmentLedger> {
        self.inner.list()
    }
}

pub fn new_cached_segment_ledger_store(
    inner: Rc<dyn SegmentLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SegmentLedgerStore> {
    Rc::new(CachedSegmentLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SegmentLedger>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects segment ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLedgerQuery {
    pub title: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SegmentLedgerQuery {
    pub fn matches(&self, item: &SegmentLedger) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SegmentLedger]) -> Vec<SegmentLedger> {
        let matched = collections::filter(items, |candidate: &SegmentLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_segment_ledgers(store: Rc<dyn SegmentLedgerStore>, query: &SegmentLedgerQuery) -> Vec<SegmentLedger> {
    query.apply(&store.list())
}
