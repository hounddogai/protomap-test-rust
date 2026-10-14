use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient freeze records by code.
pub trait LenientFreezeRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientFreezeRecord>;
    fn save(&self, item: &LenientFreezeRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientFreezeRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientFreezeRecordStore {
    pub items: RefCell<BTreeMap<String, LenientFreezeRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientFreezeRecordStore for MemoryLenientFreezeRecordStore {
    fn find(&self, code: &str) -> Option<LenientFreezeRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientFreezeRecord) {
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

    fn list(&self) -> Vec<LenientFreezeRecord> {
        let mut out: Vec<LenientFreezeRecord> = Vec::new();
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

pub fn new_memory_lenient_freeze_record_store() -> Rc<dyn LenientFreezeRecordStore> {
    Rc::new(MemoryLenientFreezeRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientFreezeRecordStore {
    pub inner: Rc<dyn LenientFreezeRecordStore>,
    pub cache: RefCell<cache::TtlCache<LenientFreezeRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientFreezeRecordStore for CachedLenientFreezeRecordStore {
    fn find(&self, code: &str) -> Option<LenientFreezeRecord> {
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

    fn save(&self, item: &LenientFreezeRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientFreezeRecord> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_freeze_record_store(
    inner: Rc<dyn LenientFreezeRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientFreezeRecordStore> {
    Rc::new(CachedLenientFreezeRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientFreezeRecord>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient freeze records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeRecordQuery {
    pub category: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientFreezeRecordQuery {
    pub fn matches(&self, item: &LenientFreezeRecord) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientFreezeRecord]) -> Vec<LenientFreezeRecord> {
        let matched = collections::filter(items, |candidate: &LenientFreezeRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_freeze_records(
    store: Rc<dyn LenientFreezeRecordStore>,
    query: &LenientFreezeRecordQuery,
) -> Vec<LenientFreezeRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientFreezeRecordStore> = new_memory_lenient_freeze_record_store();
        let item = new_lenient_freeze_record("code-346", "name-774");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
