use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped peering records by code.
pub trait CappedPeeringRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedPeeringRecord>;
    fn save(&self, item: &CappedPeeringRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedPeeringRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedPeeringRecordStore {
    pub items: RefCell<BTreeMap<String, CappedPeeringRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedPeeringRecordStore for MemoryCappedPeeringRecordStore {
    fn find(&self, code: &str) -> Option<CappedPeeringRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedPeeringRecord) {
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

    fn list(&self) -> Vec<CappedPeeringRecord> {
        let mut out: Vec<CappedPeeringRecord> = Vec::new();
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

pub fn new_memory_capped_peering_record_store() -> Rc<dyn CappedPeeringRecordStore> {
    Rc::new(MemoryCappedPeeringRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedPeeringRecordStore {
    pub inner: Rc<dyn CappedPeeringRecordStore>,
    pub cache: RefCell<cache::TtlCache<CappedPeeringRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedPeeringRecordStore for CachedCappedPeeringRecordStore {
    fn find(&self, code: &str) -> Option<CappedPeeringRecord> {
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

    fn save(&self, item: &CappedPeeringRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedPeeringRecord> {
        self.inner.list()
    }
}

pub fn new_cached_capped_peering_record_store(
    inner: Rc<dyn CappedPeeringRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedPeeringRecordStore> {
    Rc::new(CachedCappedPeeringRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedPeeringRecord>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped peering records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringRecordQuery {
    pub label: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedPeeringRecordQuery {
    pub fn matches(&self, item: &CappedPeeringRecord) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedPeeringRecord]) -> Vec<CappedPeeringRecord> {
        let matched = collections::filter(items, |candidate: &CappedPeeringRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_peering_records(
    store: Rc<dyn CappedPeeringRecordStore>,
    query: &CappedPeeringRecordQuery,
) -> Vec<CappedPeeringRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedPeeringRecordStore> = new_memory_capped_peering_record_store();
        let item = new_capped_peering_record("code-554", "label-372");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
