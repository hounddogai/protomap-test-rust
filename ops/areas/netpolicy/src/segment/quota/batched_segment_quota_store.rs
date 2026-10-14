use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched segment quotas by code.
pub trait BatchedSegmentQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedSegmentQuota>;
    fn save(&self, item: &BatchedSegmentQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedSegmentQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedSegmentQuotaStore {
    pub items: RefCell<BTreeMap<String, BatchedSegmentQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedSegmentQuotaStore for MemoryBatchedSegmentQuotaStore {
    fn find(&self, code: &str) -> Option<BatchedSegmentQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedSegmentQuota) {
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

    fn list(&self) -> Vec<BatchedSegmentQuota> {
        let mut out: Vec<BatchedSegmentQuota> = Vec::new();
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

pub fn new_memory_batched_segment_quota_store() -> Rc<dyn BatchedSegmentQuotaStore> {
    Rc::new(MemoryBatchedSegmentQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedSegmentQuotaStore {
    pub inner: Rc<dyn BatchedSegmentQuotaStore>,
    pub cache: RefCell<cache::TtlCache<BatchedSegmentQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedSegmentQuotaStore for CachedBatchedSegmentQuotaStore {
    fn find(&self, code: &str) -> Option<BatchedSegmentQuota> {
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

    fn save(&self, item: &BatchedSegmentQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedSegmentQuota> {
        self.inner.list()
    }
}

pub fn new_cached_batched_segment_quota_store(
    inner: Rc<dyn BatchedSegmentQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedSegmentQuotaStore> {
    Rc::new(CachedBatchedSegmentQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedSegmentQuota>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched segment quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentQuotaQuery {
    pub title: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedSegmentQuotaQuery {
    pub fn matches(&self, item: &BatchedSegmentQuota) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedSegmentQuota]) -> Vec<BatchedSegmentQuota> {
        let matched = collections::filter(items, |candidate: &BatchedSegmentQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_segment_quotas(
    store: Rc<dyn BatchedSegmentQuotaStore>,
    query: &BatchedSegmentQuotaQuery,
) -> Vec<BatchedSegmentQuota> {
    query.apply(&store.list())
}
