use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps release ledgers by code.
pub trait ReleaseLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReleaseLedger>;
    fn save(&self, item: &ReleaseLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReleaseLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReleaseLedgerStore {
    pub items: RefCell<BTreeMap<String, ReleaseLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl ReleaseLedgerStore for MemoryReleaseLedgerStore {
    fn find(&self, code: &str) -> Option<ReleaseLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReleaseLedger) {
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

    fn list(&self) -> Vec<ReleaseLedger> {
        let mut out: Vec<ReleaseLedger> = Vec::new();
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

pub fn new_memory_release_ledger_store() -> Rc<dyn ReleaseLedgerStore> {
    Rc::new(MemoryReleaseLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReleaseLedgerStore {
    pub inner: Rc<dyn ReleaseLedgerStore>,
    pub cache: RefCell<cache::TtlCache<ReleaseLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReleaseLedgerStore for CachedReleaseLedgerStore {
    fn find(&self, code: &str) -> Option<ReleaseLedger> {
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

    fn save(&self, item: &ReleaseLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReleaseLedger> {
        self.inner.list()
    }
}

pub fn new_cached_release_ledger_store(
    inner: Rc<dyn ReleaseLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReleaseLedgerStore> {
    Rc::new(CachedReleaseLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReleaseLedger>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects release ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseLedgerQuery {
    pub category: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReleaseLedgerQuery {
    pub fn matches(&self, item: &ReleaseLedger) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReleaseLedger]) -> Vec<ReleaseLedger> {
        let matched = collections::filter(items, |candidate: &ReleaseLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_release_ledgers(store: Rc<dyn ReleaseLedgerStore>, query: &ReleaseLedgerQuery) -> Vec<ReleaseLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReleaseLedgerStore> = new_memory_release_ledger_store();
        let item = new_release_ledger("code-380", "title-611");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
