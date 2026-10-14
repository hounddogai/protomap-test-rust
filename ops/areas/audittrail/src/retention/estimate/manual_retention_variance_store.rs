use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual retention variances by code.
pub trait ManualRetentionVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualRetentionVariance>;
    fn save(&self, item: &ManualRetentionVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualRetentionVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualRetentionVarianceStore {
    pub items: RefCell<BTreeMap<String, ManualRetentionVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualRetentionVarianceStore for MemoryManualRetentionVarianceStore {
    fn find(&self, code: &str) -> Option<ManualRetentionVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualRetentionVariance) {
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

    fn list(&self) -> Vec<ManualRetentionVariance> {
        let mut out: Vec<ManualRetentionVariance> = Vec::new();
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

pub fn new_memory_manual_retention_variance_store() -> Rc<dyn ManualRetentionVarianceStore> {
    Rc::new(MemoryManualRetentionVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualRetentionVarianceStore {
    pub inner: Rc<dyn ManualRetentionVarianceStore>,
    pub cache: RefCell<cache::TtlCache<ManualRetentionVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualRetentionVarianceStore for CachedManualRetentionVarianceStore {
    fn find(&self, code: &str) -> Option<ManualRetentionVariance> {
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

    fn save(&self, item: &ManualRetentionVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualRetentionVariance> {
        self.inner.list()
    }
}

pub fn new_cached_manual_retention_variance_store(
    inner: Rc<dyn ManualRetentionVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualRetentionVarianceStore> {
    Rc::new(CachedManualRetentionVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualRetentionVariance>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual retention variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionVarianceQuery {
    pub title: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualRetentionVarianceQuery {
    pub fn matches(&self, item: &ManualRetentionVariance) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualRetentionVariance]) -> Vec<ManualRetentionVariance> {
        let matched = collections::filter(items, |candidate: &ManualRetentionVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_retention_variances(
    store: Rc<dyn ManualRetentionVarianceStore>,
    query: &ManualRetentionVarianceQuery,
) -> Vec<ManualRetentionVariance> {
    query.apply(&store.list())
}
