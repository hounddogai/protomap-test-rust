use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive region pin ledgers by code.
pub trait AdaptiveRegionPinLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinLedger>;
    fn save(&self, item: &AdaptiveRegionPinLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveRegionPinLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveRegionPinLedgerStore {
    pub items: RefCell<BTreeMap<String, AdaptiveRegionPinLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveRegionPinLedgerStore for MemoryAdaptiveRegionPinLedgerStore {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveRegionPinLedger) {
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

    fn list(&self) -> Vec<AdaptiveRegionPinLedger> {
        let mut out: Vec<AdaptiveRegionPinLedger> = Vec::new();
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

pub fn new_memory_adaptive_region_pin_ledger_store() -> Rc<dyn AdaptiveRegionPinLedgerStore> {
    Rc::new(MemoryAdaptiveRegionPinLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveRegionPinLedgerStore {
    pub inner: Rc<dyn AdaptiveRegionPinLedgerStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveRegionPinLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveRegionPinLedgerStore for CachedAdaptiveRegionPinLedgerStore {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinLedger> {
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

    fn save(&self, item: &AdaptiveRegionPinLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveRegionPinLedger> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_region_pin_ledger_store(
    inner: Rc<dyn AdaptiveRegionPinLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveRegionPinLedgerStore> {
    Rc::new(CachedAdaptiveRegionPinLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveRegionPinLedger>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive region pin ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinLedgerQuery {
    pub segment: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveRegionPinLedgerQuery {
    pub fn matches(&self, item: &AdaptiveRegionPinLedger) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveRegionPinLedger]) -> Vec<AdaptiveRegionPinLedger> {
        let matched = collections::filter(items, |candidate: &AdaptiveRegionPinLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_region_pin_ledgers(
    store: Rc<dyn AdaptiveRegionPinLedgerStore>,
    query: &AdaptiveRegionPinLedgerQuery,
) -> Vec<AdaptiveRegionPinLedger> {
    query.apply(&store.list())
}
