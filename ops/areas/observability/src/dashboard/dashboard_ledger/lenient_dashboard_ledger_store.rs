use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient dashboard ledgers by code.
pub trait LenientDashboardLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientDashboardLedger>;
    fn save(&self, item: &LenientDashboardLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientDashboardLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientDashboardLedgerStore {
    pub items: RefCell<BTreeMap<String, LenientDashboardLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientDashboardLedgerStore for MemoryLenientDashboardLedgerStore {
    fn find(&self, code: &str) -> Option<LenientDashboardLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientDashboardLedger) {
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

    fn list(&self) -> Vec<LenientDashboardLedger> {
        let mut out: Vec<LenientDashboardLedger> = Vec::new();
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

pub fn new_memory_lenient_dashboard_ledger_store() -> Rc<dyn LenientDashboardLedgerStore> {
    Rc::new(MemoryLenientDashboardLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientDashboardLedgerStore {
    pub inner: Rc<dyn LenientDashboardLedgerStore>,
    pub cache: RefCell<cache::TtlCache<LenientDashboardLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientDashboardLedgerStore for CachedLenientDashboardLedgerStore {
    fn find(&self, code: &str) -> Option<LenientDashboardLedger> {
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

    fn save(&self, item: &LenientDashboardLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientDashboardLedger> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_dashboard_ledger_store(
    inner: Rc<dyn LenientDashboardLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientDashboardLedgerStore> {
    Rc::new(CachedLenientDashboardLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientDashboardLedger>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient dashboard ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardLedgerQuery {
    pub label: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientDashboardLedgerQuery {
    pub fn matches(&self, item: &LenientDashboardLedger) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientDashboardLedger]) -> Vec<LenientDashboardLedger> {
        let matched = collections::filter(items, |candidate: &LenientDashboardLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_dashboard_ledgers(
    store: Rc<dyn LenientDashboardLedgerStore>,
    query: &LenientDashboardLedgerQuery,
) -> Vec<LenientDashboardLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientDashboardLedgerStore> = new_memory_lenient_dashboard_ledger_store();
        let item = new_lenient_dashboard_ledger("code-12", "label-532");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
