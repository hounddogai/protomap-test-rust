use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps segment entries by code.
pub trait SegmentEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SegmentEntry>;
    fn save(&self, item: &SegmentEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SegmentEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySegmentEntryStore {
    pub items: RefCell<BTreeMap<String, SegmentEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl SegmentEntryStore for MemorySegmentEntryStore {
    fn find(&self, code: &str) -> Option<SegmentEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SegmentEntry) {
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

    fn list(&self) -> Vec<SegmentEntry> {
        let mut out: Vec<SegmentEntry> = Vec::new();
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

pub fn new_memory_segment_entry_store() -> Rc<dyn SegmentEntryStore> {
    Rc::new(MemorySegmentEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSegmentEntryStore {
    pub inner: Rc<dyn SegmentEntryStore>,
    pub cache: RefCell<cache::TtlCache<SegmentEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SegmentEntryStore for CachedSegmentEntryStore {
    fn find(&self, code: &str) -> Option<SegmentEntry> {
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

    fn save(&self, item: &SegmentEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SegmentEntry> {
        self.inner.list()
    }
}

pub fn new_cached_segment_entry_store(
    inner: Rc<dyn SegmentEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SegmentEntryStore> {
    Rc::new(CachedSegmentEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SegmentEntry>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects segment entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEntryQuery {
    pub segment: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SegmentEntryQuery {
    pub fn matches(&self, item: &SegmentEntry) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SegmentEntry]) -> Vec<SegmentEntry> {
        let matched = collections::filter(items, |candidate: &SegmentEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_segment_entries(store: Rc<dyn SegmentEntryStore>, query: &SegmentEntryQuery) -> Vec<SegmentEntry> {
    query.apply(&store.list())
}
