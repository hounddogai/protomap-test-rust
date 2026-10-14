use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps egress records by code.
pub trait EgressRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EgressRecord>;
    fn save(&self, item: &EgressRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EgressRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEgressRecordStore {
    pub items: RefCell<BTreeMap<String, EgressRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl EgressRecordStore for MemoryEgressRecordStore {
    fn find(&self, code: &str) -> Option<EgressRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EgressRecord) {
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

    fn list(&self) -> Vec<EgressRecord> {
        let mut out: Vec<EgressRecord> = Vec::new();
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

pub fn new_memory_egress_record_store() -> Rc<dyn EgressRecordStore> {
    Rc::new(MemoryEgressRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEgressRecordStore {
    pub inner: Rc<dyn EgressRecordStore>,
    pub cache: RefCell<cache::TtlCache<EgressRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EgressRecordStore for CachedEgressRecordStore {
    fn find(&self, code: &str) -> Option<EgressRecord> {
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

    fn save(&self, item: &EgressRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EgressRecord> {
        self.inner.list()
    }
}

pub fn new_cached_egress_record_store(
    inner: Rc<dyn EgressRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EgressRecordStore> {
    Rc::new(CachedEgressRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EgressRecord>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects egress records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressRecordQuery {
    pub country: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EgressRecordQuery {
    pub fn matches(&self, item: &EgressRecord) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[EgressRecord]) -> Vec<EgressRecord> {
        let matched = collections::filter(items, |candidate: &EgressRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_egress_records(store: Rc<dyn EgressRecordStore>, query: &EgressRecordQuery) -> Vec<EgressRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EgressRecordStore> = new_memory_egress_record_store();
        let item = new_egress_record("code-502", "title-318");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
