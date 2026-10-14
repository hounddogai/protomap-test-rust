use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched identity rollups by code.
pub trait BatchedIdentityRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedIdentityRollup>;
    fn save(&self, item: &BatchedIdentityRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedIdentityRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedIdentityRollupStore {
    pub items: RefCell<BTreeMap<String, BatchedIdentityRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedIdentityRollupStore for MemoryBatchedIdentityRollupStore {
    fn find(&self, code: &str) -> Option<BatchedIdentityRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedIdentityRollup) {
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

    fn list(&self) -> Vec<BatchedIdentityRollup> {
        let mut out: Vec<BatchedIdentityRollup> = Vec::new();
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

pub fn new_memory_batched_identity_rollup_store() -> Rc<dyn BatchedIdentityRollupStore> {
    Rc::new(MemoryBatchedIdentityRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedIdentityRollupStore {
    pub inner: Rc<dyn BatchedIdentityRollupStore>,
    pub cache: RefCell<cache::TtlCache<BatchedIdentityRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedIdentityRollupStore for CachedBatchedIdentityRollupStore {
    fn find(&self, code: &str) -> Option<BatchedIdentityRollup> {
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

    fn save(&self, item: &BatchedIdentityRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedIdentityRollup> {
        self.inner.list()
    }
}

pub fn new_cached_batched_identity_rollup_store(
    inner: Rc<dyn BatchedIdentityRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedIdentityRollupStore> {
    Rc::new(CachedBatchedIdentityRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedIdentityRollup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched identity rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityRollupQuery {
    pub title: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedIdentityRollupQuery {
    pub fn matches(&self, item: &BatchedIdentityRollup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedIdentityRollup]) -> Vec<BatchedIdentityRollup> {
        let matched = collections::filter(items, |candidate: &BatchedIdentityRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_identity_rollups(
    store: Rc<dyn BatchedIdentityRollupStore>,
    query: &BatchedIdentityRollupQuery,
) -> Vec<BatchedIdentityRollup> {
    query.apply(&store.list())
}
