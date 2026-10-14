use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched allowlist variances by code.
pub trait BatchedAllowlistVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedAllowlistVariance>;
    fn save(&self, item: &BatchedAllowlistVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedAllowlistVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedAllowlistVarianceStore {
    pub items: RefCell<BTreeMap<String, BatchedAllowlistVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedAllowlistVarianceStore for MemoryBatchedAllowlistVarianceStore {
    fn find(&self, code: &str) -> Option<BatchedAllowlistVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedAllowlistVariance) {
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

    fn list(&self) -> Vec<BatchedAllowlistVariance> {
        let mut out: Vec<BatchedAllowlistVariance> = Vec::new();
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

pub fn new_memory_batched_allowlist_variance_store() -> Rc<dyn BatchedAllowlistVarianceStore> {
    Rc::new(MemoryBatchedAllowlistVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedAllowlistVarianceStore {
    pub inner: Rc<dyn BatchedAllowlistVarianceStore>,
    pub cache: RefCell<cache::TtlCache<BatchedAllowlistVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedAllowlistVarianceStore for CachedBatchedAllowlistVarianceStore {
    fn find(&self, code: &str) -> Option<BatchedAllowlistVariance> {
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

    fn save(&self, item: &BatchedAllowlistVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedAllowlistVariance> {
        self.inner.list()
    }
}

pub fn new_cached_batched_allowlist_variance_store(
    inner: Rc<dyn BatchedAllowlistVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedAllowlistVarianceStore> {
    Rc::new(CachedBatchedAllowlistVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedAllowlistVariance>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched allowlist variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistVarianceQuery {
    pub locale: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedAllowlistVarianceQuery {
    pub fn matches(&self, item: &BatchedAllowlistVariance) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
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

    pub fn apply(&self, items: &[BatchedAllowlistVariance]) -> Vec<BatchedAllowlistVariance> {
        let matched = collections::filter(items, |candidate: &BatchedAllowlistVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_allowlist_variances(
    store: Rc<dyn BatchedAllowlistVarianceStore>,
    query: &BatchedAllowlistVarianceQuery,
) -> Vec<BatchedAllowlistVariance> {
    query.apply(&store.list())
}
