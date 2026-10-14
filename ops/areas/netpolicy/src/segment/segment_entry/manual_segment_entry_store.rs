use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual segment entries by code.
pub trait ManualSegmentEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualSegmentEntry>;
    fn save(&self, item: &ManualSegmentEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualSegmentEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualSegmentEntryStore {
    pub items: RefCell<BTreeMap<String, ManualSegmentEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualSegmentEntryStore for MemoryManualSegmentEntryStore {
    fn find(&self, code: &str) -> Option<ManualSegmentEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualSegmentEntry) {
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

    fn list(&self) -> Vec<ManualSegmentEntry> {
        let mut out: Vec<ManualSegmentEntry> = Vec::new();
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

pub fn new_memory_manual_segment_entry_store() -> Rc<dyn ManualSegmentEntryStore> {
    Rc::new(MemoryManualSegmentEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualSegmentEntryStore {
    pub inner: Rc<dyn ManualSegmentEntryStore>,
    pub cache: RefCell<cache::TtlCache<ManualSegmentEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualSegmentEntryStore for CachedManualSegmentEntryStore {
    fn find(&self, code: &str) -> Option<ManualSegmentEntry> {
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

    fn save(&self, item: &ManualSegmentEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualSegmentEntry> {
        self.inner.list()
    }
}

pub fn new_cached_manual_segment_entry_store(
    inner: Rc<dyn ManualSegmentEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualSegmentEntryStore> {
    Rc::new(CachedManualSegmentEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualSegmentEntry>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual segment entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSegmentEntryQuery {
    pub segment: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualSegmentEntryQuery {
    pub fn matches(&self, item: &ManualSegmentEntry) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[ManualSegmentEntry]) -> Vec<ManualSegmentEntry> {
        let matched = collections::filter(items, |candidate: &ManualSegmentEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_segment_entries(
    store: Rc<dyn ManualSegmentEntryStore>,
    query: &ManualSegmentEntryQuery,
) -> Vec<ManualSegmentEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualSegmentEntryStore> = new_memory_manual_segment_entry_store();
        let item = new_manual_segment_entry("code-186", "name-224");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
