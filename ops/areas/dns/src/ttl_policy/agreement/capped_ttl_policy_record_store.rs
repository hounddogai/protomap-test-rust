use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped ttl policy records by code.
pub trait CappedTtlPolicyRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedTtlPolicyRecord>;
    fn save(&self, item: &CappedTtlPolicyRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedTtlPolicyRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedTtlPolicyRecordStore {
    pub items: RefCell<BTreeMap<String, CappedTtlPolicyRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedTtlPolicyRecordStore for MemoryCappedTtlPolicyRecordStore {
    fn find(&self, code: &str) -> Option<CappedTtlPolicyRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedTtlPolicyRecord) {
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

    fn list(&self) -> Vec<CappedTtlPolicyRecord> {
        let mut out: Vec<CappedTtlPolicyRecord> = Vec::new();
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

pub fn new_memory_capped_ttl_policy_record_store() -> Rc<dyn CappedTtlPolicyRecordStore> {
    Rc::new(MemoryCappedTtlPolicyRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedTtlPolicyRecordStore {
    pub inner: Rc<dyn CappedTtlPolicyRecordStore>,
    pub cache: RefCell<cache::TtlCache<CappedTtlPolicyRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedTtlPolicyRecordStore for CachedCappedTtlPolicyRecordStore {
    fn find(&self, code: &str) -> Option<CappedTtlPolicyRecord> {
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

    fn save(&self, item: &CappedTtlPolicyRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedTtlPolicyRecord> {
        self.inner.list()
    }
}

pub fn new_cached_capped_ttl_policy_record_store(
    inner: Rc<dyn CappedTtlPolicyRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedTtlPolicyRecordStore> {
    Rc::new(CachedCappedTtlPolicyRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedTtlPolicyRecord>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped ttl policy records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTtlPolicyRecordQuery {
    pub title: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedTtlPolicyRecordQuery {
    pub fn matches(&self, item: &CappedTtlPolicyRecord) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedTtlPolicyRecord]) -> Vec<CappedTtlPolicyRecord> {
        let matched = collections::filter(items, |candidate: &CappedTtlPolicyRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_ttl_policy_records(
    store: Rc<dyn CappedTtlPolicyRecordStore>,
    query: &CappedTtlPolicyRecordQuery,
) -> Vec<CappedTtlPolicyRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedTtlPolicyRecordStore> = new_memory_capped_ttl_policy_record_store();
        let item = new_capped_ttl_policy_record("code-704", "title-716");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
