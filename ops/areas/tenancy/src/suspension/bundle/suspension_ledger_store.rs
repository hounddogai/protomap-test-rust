use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension ledgers by code.
pub trait SuspensionLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionLedger>;
    fn save(&self, item: &SuspensionLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionLedgerStore {
    pub items: RefCell<BTreeMap<String, SuspensionLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionLedgerStore for MemorySuspensionLedgerStore {
    fn find(&self, code: &str) -> Option<SuspensionLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionLedger) {
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

    fn list(&self) -> Vec<SuspensionLedger> {
        let mut out: Vec<SuspensionLedger> = Vec::new();
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

pub fn new_memory_suspension_ledger_store() -> Rc<dyn SuspensionLedgerStore> {
    Rc::new(MemorySuspensionLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionLedgerStore {
    pub inner: Rc<dyn SuspensionLedgerStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionLedgerStore for CachedSuspensionLedgerStore {
    fn find(&self, code: &str) -> Option<SuspensionLedger> {
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

    fn save(&self, item: &SuspensionLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionLedger> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_ledger_store(
    inner: Rc<dyn SuspensionLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionLedgerStore> {
    Rc::new(CachedSuspensionLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionLedger>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionLedgerQuery {
    pub label: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionLedgerQuery {
    pub fn matches(&self, item: &SuspensionLedger) -> bool {
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

    pub fn apply(&self, items: &[SuspensionLedger]) -> Vec<SuspensionLedger> {
        let matched = collections::filter(items, |candidate: &SuspensionLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_ledgers(
    store: Rc<dyn SuspensionLedgerStore>,
    query: &SuspensionLedgerQuery,
) -> Vec<SuspensionLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SuspensionLedgerStore> = new_memory_suspension_ledger_store();
        let item = new_suspension_ledger("code-365", "label-56");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
