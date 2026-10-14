use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched freeze segments by code.
pub trait BatchedFreezeSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedFreezeSegment>;
    fn save(&self, item: &BatchedFreezeSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedFreezeSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedFreezeSegmentStore {
    pub items: RefCell<BTreeMap<String, BatchedFreezeSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedFreezeSegmentStore for MemoryBatchedFreezeSegmentStore {
    fn find(&self, code: &str) -> Option<BatchedFreezeSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedFreezeSegment) {
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

    fn list(&self) -> Vec<BatchedFreezeSegment> {
        let mut out: Vec<BatchedFreezeSegment> = Vec::new();
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

pub fn new_memory_batched_freeze_segment_store() -> Rc<dyn BatchedFreezeSegmentStore> {
    Rc::new(MemoryBatchedFreezeSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedFreezeSegmentStore {
    pub inner: Rc<dyn BatchedFreezeSegmentStore>,
    pub cache: RefCell<cache::TtlCache<BatchedFreezeSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedFreezeSegmentStore for CachedBatchedFreezeSegmentStore {
    fn find(&self, code: &str) -> Option<BatchedFreezeSegment> {
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

    fn save(&self, item: &BatchedFreezeSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedFreezeSegment> {
        self.inner.list()
    }
}

pub fn new_cached_batched_freeze_segment_store(
    inner: Rc<dyn BatchedFreezeSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedFreezeSegmentStore> {
    Rc::new(CachedBatchedFreezeSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedFreezeSegment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched freeze segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFreezeSegmentQuery {
    pub country: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedFreezeSegmentQuery {
    pub fn matches(&self, item: &BatchedFreezeSegment) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[BatchedFreezeSegment]) -> Vec<BatchedFreezeSegment> {
        let matched = collections::filter(items, |candidate: &BatchedFreezeSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_freeze_segments(
    store: Rc<dyn BatchedFreezeSegmentStore>,
    query: &BatchedFreezeSegmentQuery,
) -> Vec<BatchedFreezeSegment> {
    query.apply(&store.list())
}
