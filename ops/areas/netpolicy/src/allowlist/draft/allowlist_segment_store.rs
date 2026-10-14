use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps allowlist segments by code.
pub trait AllowlistSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AllowlistSegment>;
    fn save(&self, item: &AllowlistSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AllowlistSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAllowlistSegmentStore {
    pub items: RefCell<BTreeMap<String, AllowlistSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl AllowlistSegmentStore for MemoryAllowlistSegmentStore {
    fn find(&self, code: &str) -> Option<AllowlistSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AllowlistSegment) {
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

    fn list(&self) -> Vec<AllowlistSegment> {
        let mut out: Vec<AllowlistSegment> = Vec::new();
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

pub fn new_memory_allowlist_segment_store() -> Rc<dyn AllowlistSegmentStore> {
    Rc::new(MemoryAllowlistSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAllowlistSegmentStore {
    pub inner: Rc<dyn AllowlistSegmentStore>,
    pub cache: RefCell<cache::TtlCache<AllowlistSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AllowlistSegmentStore for CachedAllowlistSegmentStore {
    fn find(&self, code: &str) -> Option<AllowlistSegment> {
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

    fn save(&self, item: &AllowlistSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AllowlistSegment> {
        self.inner.list()
    }
}

pub fn new_cached_allowlist_segment_store(
    inner: Rc<dyn AllowlistSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AllowlistSegmentStore> {
    Rc::new(CachedAllowlistSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AllowlistSegment>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects allowlist segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSegmentQuery {
    pub label: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AllowlistSegmentQuery {
    pub fn matches(&self, item: &AllowlistSegment) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AllowlistSegment]) -> Vec<AllowlistSegment> {
        let matched = collections::filter(items, |candidate: &AllowlistSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_allowlist_segments(
    store: Rc<dyn AllowlistSegmentStore>,
    query: &AllowlistSegmentQuery,
) -> Vec<AllowlistSegment> {
    query.apply(&store.list())
}
