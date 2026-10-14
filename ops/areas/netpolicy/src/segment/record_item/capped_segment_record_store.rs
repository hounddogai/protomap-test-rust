use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped segment records by code.
pub trait CappedSegmentRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedSegmentRecord>;
    fn save(&self, item: &CappedSegmentRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedSegmentRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedSegmentRecordStore {
    pub items: RefCell<BTreeMap<String, CappedSegmentRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedSegmentRecordStore for MemoryCappedSegmentRecordStore {
    fn find(&self, code: &str) -> Option<CappedSegmentRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedSegmentRecord) {
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

    fn list(&self) -> Vec<CappedSegmentRecord> {
        let mut out: Vec<CappedSegmentRecord> = Vec::new();
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

pub fn new_memory_capped_segment_record_store() -> Rc<dyn CappedSegmentRecordStore> {
    Rc::new(MemoryCappedSegmentRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedSegmentRecordStore {
    pub inner: Rc<dyn CappedSegmentRecordStore>,
    pub cache: RefCell<cache::TtlCache<CappedSegmentRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedSegmentRecordStore for CachedCappedSegmentRecordStore {
    fn find(&self, code: &str) -> Option<CappedSegmentRecord> {
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

    fn save(&self, item: &CappedSegmentRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedSegmentRecord> {
        self.inner.list()
    }
}

pub fn new_cached_capped_segment_record_store(
    inner: Rc<dyn CappedSegmentRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedSegmentRecordStore> {
    Rc::new(CachedCappedSegmentRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedSegmentRecord>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped segment records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentRecordQuery {
    pub label: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedSegmentRecordQuery {
    pub fn matches(&self, item: &CappedSegmentRecord) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedSegmentRecord]) -> Vec<CappedSegmentRecord> {
        let matched = collections::filter(items, |candidate: &CappedSegmentRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_segment_records(
    store: Rc<dyn CappedSegmentRecordStore>,
    query: &CappedSegmentRecordQuery,
) -> Vec<CappedSegmentRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedSegmentRecordStore> = new_memory_capped_segment_record_store();
        let item = new_capped_segment_record("code-820", "label-922");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
