use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched rollout ledgers by code.
pub trait BatchedRolloutLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedRolloutLedger>;
    fn save(&self, item: &BatchedRolloutLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedRolloutLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedRolloutLedgerStore {
    pub items: RefCell<BTreeMap<String, BatchedRolloutLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedRolloutLedgerStore for MemoryBatchedRolloutLedgerStore {
    fn find(&self, code: &str) -> Option<BatchedRolloutLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedRolloutLedger) {
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

    fn list(&self) -> Vec<BatchedRolloutLedger> {
        let mut out: Vec<BatchedRolloutLedger> = Vec::new();
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

pub fn new_memory_batched_rollout_ledger_store() -> Rc<dyn BatchedRolloutLedgerStore> {
    Rc::new(MemoryBatchedRolloutLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedRolloutLedgerStore {
    pub inner: Rc<dyn BatchedRolloutLedgerStore>,
    pub cache: RefCell<cache::TtlCache<BatchedRolloutLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedRolloutLedgerStore for CachedBatchedRolloutLedgerStore {
    fn find(&self, code: &str) -> Option<BatchedRolloutLedger> {
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

    fn save(&self, item: &BatchedRolloutLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedRolloutLedger> {
        self.inner.list()
    }
}

pub fn new_cached_batched_rollout_ledger_store(
    inner: Rc<dyn BatchedRolloutLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedRolloutLedgerStore> {
    Rc::new(CachedBatchedRolloutLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedRolloutLedger>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched rollout ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRolloutLedgerQuery {
    pub namespace: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedRolloutLedgerQuery {
    pub fn matches(&self, item: &BatchedRolloutLedger) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedRolloutLedger]) -> Vec<BatchedRolloutLedger> {
        let matched = collections::filter(items, |candidate: &BatchedRolloutLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_rollout_ledgers(
    store: Rc<dyn BatchedRolloutLedgerStore>,
    query: &BatchedRolloutLedgerQuery,
) -> Vec<BatchedRolloutLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedRolloutLedgerStore> = new_memory_batched_rollout_ledger_store();
        let item = new_batched_rollout_ledger("code-931", "name-284");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
