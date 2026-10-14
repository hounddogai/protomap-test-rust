use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient egress records by code.
pub trait LenientEgressRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientEgressRecord>;
    fn save(&self, item: &LenientEgressRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientEgressRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientEgressRecordStore {
    pub items: RefCell<BTreeMap<String, LenientEgressRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientEgressRecordStore for MemoryLenientEgressRecordStore {
    fn find(&self, code: &str) -> Option<LenientEgressRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientEgressRecord) {
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

    fn list(&self) -> Vec<LenientEgressRecord> {
        let mut out: Vec<LenientEgressRecord> = Vec::new();
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

pub fn new_memory_lenient_egress_record_store() -> Rc<dyn LenientEgressRecordStore> {
    Rc::new(MemoryLenientEgressRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientEgressRecordStore {
    pub inner: Rc<dyn LenientEgressRecordStore>,
    pub cache: RefCell<cache::TtlCache<LenientEgressRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientEgressRecordStore for CachedLenientEgressRecordStore {
    fn find(&self, code: &str) -> Option<LenientEgressRecord> {
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

    fn save(&self, item: &LenientEgressRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientEgressRecord> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_egress_record_store(
    inner: Rc<dyn LenientEgressRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientEgressRecordStore> {
    Rc::new(CachedLenientEgressRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientEgressRecord>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient egress records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressRecordQuery {
    pub category: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientEgressRecordQuery {
    pub fn matches(&self, item: &LenientEgressRecord) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[LenientEgressRecord]) -> Vec<LenientEgressRecord> {
        let matched = collections::filter(items, |candidate: &LenientEgressRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_egress_records(
    store: Rc<dyn LenientEgressRecordStore>,
    query: &LenientEgressRecordQuery,
) -> Vec<LenientEgressRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientEgressRecordStore> = new_memory_lenient_egress_record_store();
        let item = new_lenient_egress_record("code-225", "name-8");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
