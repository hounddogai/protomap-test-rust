use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered rollout segments by code.
pub trait TieredRolloutSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredRolloutSegment>;
    fn save(&self, item: &TieredRolloutSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredRolloutSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredRolloutSegmentStore {
    pub items: RefCell<BTreeMap<String, TieredRolloutSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredRolloutSegmentStore for MemoryTieredRolloutSegmentStore {
    fn find(&self, code: &str) -> Option<TieredRolloutSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredRolloutSegment) {
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

    fn list(&self) -> Vec<TieredRolloutSegment> {
        let mut out: Vec<TieredRolloutSegment> = Vec::new();
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

pub fn new_memory_tiered_rollout_segment_store() -> Rc<dyn TieredRolloutSegmentStore> {
    Rc::new(MemoryTieredRolloutSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredRolloutSegmentStore {
    pub inner: Rc<dyn TieredRolloutSegmentStore>,
    pub cache: RefCell<cache::TtlCache<TieredRolloutSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredRolloutSegmentStore for CachedTieredRolloutSegmentStore {
    fn find(&self, code: &str) -> Option<TieredRolloutSegment> {
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

    fn save(&self, item: &TieredRolloutSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredRolloutSegment> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_rollout_segment_store(
    inner: Rc<dyn TieredRolloutSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredRolloutSegmentStore> {
    Rc::new(CachedTieredRolloutSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredRolloutSegment>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered rollout segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSegmentQuery {
    pub label: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredRolloutSegmentQuery {
    pub fn matches(&self, item: &TieredRolloutSegment) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[TieredRolloutSegment]) -> Vec<TieredRolloutSegment> {
        let matched = collections::filter(items, |candidate: &TieredRolloutSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_rollout_segments(
    store: Rc<dyn TieredRolloutSegmentStore>,
    query: &TieredRolloutSegmentQuery,
) -> Vec<TieredRolloutSegment> {
    query.apply(&store.list())
}
