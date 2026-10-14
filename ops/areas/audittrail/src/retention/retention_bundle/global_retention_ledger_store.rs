use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global retention ledgers by code.
pub trait GlobalRetentionLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalRetentionLedger>;
    fn save(&self, item: &GlobalRetentionLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalRetentionLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalRetentionLedgerStore {
    pub items: RefCell<BTreeMap<String, GlobalRetentionLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalRetentionLedgerStore for MemoryGlobalRetentionLedgerStore {
    fn find(&self, code: &str) -> Option<GlobalRetentionLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalRetentionLedger) {
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

    fn list(&self) -> Vec<GlobalRetentionLedger> {
        let mut out: Vec<GlobalRetentionLedger> = Vec::new();
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

pub fn new_memory_global_retention_ledger_store() -> Rc<dyn GlobalRetentionLedgerStore> {
    Rc::new(MemoryGlobalRetentionLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalRetentionLedgerStore {
    pub inner: Rc<dyn GlobalRetentionLedgerStore>,
    pub cache: RefCell<cache::TtlCache<GlobalRetentionLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalRetentionLedgerStore for CachedGlobalRetentionLedgerStore {
    fn find(&self, code: &str) -> Option<GlobalRetentionLedger> {
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

    fn save(&self, item: &GlobalRetentionLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalRetentionLedger> {
        self.inner.list()
    }
}

pub fn new_cached_global_retention_ledger_store(
    inner: Rc<dyn GlobalRetentionLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalRetentionLedgerStore> {
    Rc::new(CachedGlobalRetentionLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalRetentionLedger>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global retention ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionLedgerQuery {
    pub label: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalRetentionLedgerQuery {
    pub fn matches(&self, item: &GlobalRetentionLedger) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalRetentionLedger]) -> Vec<GlobalRetentionLedger> {
        let matched = collections::filter(items, |candidate: &GlobalRetentionLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_retention_ledgers(
    store: Rc<dyn GlobalRetentionLedgerStore>,
    query: &GlobalRetentionLedgerQuery,
) -> Vec<GlobalRetentionLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalRetentionLedgerStore> = new_memory_global_retention_ledger_store();
        let item = new_global_retention_ledger("code-615", "label-587");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
