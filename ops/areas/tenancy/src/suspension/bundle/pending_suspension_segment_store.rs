use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending suspension segments by code.
pub trait PendingSuspensionSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingSuspensionSegment>;
    fn save(&self, item: &PendingSuspensionSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingSuspensionSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingSuspensionSegmentStore {
    pub items: RefCell<BTreeMap<String, PendingSuspensionSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingSuspensionSegmentStore for MemoryPendingSuspensionSegmentStore {
    fn find(&self, code: &str) -> Option<PendingSuspensionSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingSuspensionSegment) {
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

    fn list(&self) -> Vec<PendingSuspensionSegment> {
        let mut out: Vec<PendingSuspensionSegment> = Vec::new();
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

pub fn new_memory_pending_suspension_segment_store() -> Rc<dyn PendingSuspensionSegmentStore> {
    Rc::new(MemoryPendingSuspensionSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingSuspensionSegmentStore {
    pub inner: Rc<dyn PendingSuspensionSegmentStore>,
    pub cache: RefCell<cache::TtlCache<PendingSuspensionSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingSuspensionSegmentStore for CachedPendingSuspensionSegmentStore {
    fn find(&self, code: &str) -> Option<PendingSuspensionSegment> {
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

    fn save(&self, item: &PendingSuspensionSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingSuspensionSegment> {
        self.inner.list()
    }
}

pub fn new_cached_pending_suspension_segment_store(
    inner: Rc<dyn PendingSuspensionSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingSuspensionSegmentStore> {
    Rc::new(CachedPendingSuspensionSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingSuspensionSegment>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending suspension segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionSegmentQuery {
    pub title: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingSuspensionSegmentQuery {
    pub fn matches(&self, item: &PendingSuspensionSegment) -> bool {
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

    pub fn apply(&self, items: &[PendingSuspensionSegment]) -> Vec<PendingSuspensionSegment> {
        let matched = collections::filter(items, |candidate: &PendingSuspensionSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_suspension_segments(
    store: Rc<dyn PendingSuspensionSegmentStore>,
    query: &PendingSuspensionSegmentQuery,
) -> Vec<PendingSuspensionSegment> {
    query.apply(&store.list())
}
