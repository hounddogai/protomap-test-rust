use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped segment estimates by code.
pub trait CappedSegmentEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedSegmentEstimate>;
    fn save(&self, item: &CappedSegmentEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedSegmentEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedSegmentEstimateStore {
    pub items: RefCell<BTreeMap<String, CappedSegmentEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedSegmentEstimateStore for MemoryCappedSegmentEstimateStore {
    fn find(&self, code: &str) -> Option<CappedSegmentEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedSegmentEstimate) {
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

    fn list(&self) -> Vec<CappedSegmentEstimate> {
        let mut out: Vec<CappedSegmentEstimate> = Vec::new();
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

pub fn new_memory_capped_segment_estimate_store() -> Rc<dyn CappedSegmentEstimateStore> {
    Rc::new(MemoryCappedSegmentEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedSegmentEstimateStore {
    pub inner: Rc<dyn CappedSegmentEstimateStore>,
    pub cache: RefCell<cache::TtlCache<CappedSegmentEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedSegmentEstimateStore for CachedCappedSegmentEstimateStore {
    fn find(&self, code: &str) -> Option<CappedSegmentEstimate> {
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

    fn save(&self, item: &CappedSegmentEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedSegmentEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_capped_segment_estimate_store(
    inner: Rc<dyn CappedSegmentEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedSegmentEstimateStore> {
    Rc::new(CachedCappedSegmentEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedSegmentEstimate>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped segment estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentEstimateQuery {
    pub region: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedSegmentEstimateQuery {
    pub fn matches(&self, item: &CappedSegmentEstimate) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedSegmentEstimate]) -> Vec<CappedSegmentEstimate> {
        let matched = collections::filter(items, |candidate: &CappedSegmentEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_segment_estimates(
    store: Rc<dyn CappedSegmentEstimateStore>,
    query: &CappedSegmentEstimateQuery,
) -> Vec<CappedSegmentEstimate> {
    query.apply(&store.list())
}
