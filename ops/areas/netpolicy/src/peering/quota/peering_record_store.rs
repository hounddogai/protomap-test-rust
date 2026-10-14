use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering records by code.
pub trait PeeringRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringRecord>;
    fn save(&self, item: &PeeringRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringRecordStore {
    pub items: RefCell<BTreeMap<String, PeeringRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringRecordStore for MemoryPeeringRecordStore {
    fn find(&self, code: &str) -> Option<PeeringRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringRecord) {
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

    fn list(&self) -> Vec<PeeringRecord> {
        let mut out: Vec<PeeringRecord> = Vec::new();
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

pub fn new_memory_peering_record_store() -> Rc<dyn PeeringRecordStore> {
    Rc::new(MemoryPeeringRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringRecordStore {
    pub inner: Rc<dyn PeeringRecordStore>,
    pub cache: RefCell<cache::TtlCache<PeeringRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringRecordStore for CachedPeeringRecordStore {
    fn find(&self, code: &str) -> Option<PeeringRecord> {
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

    fn save(&self, item: &PeeringRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringRecord> {
        self.inner.list()
    }
}

pub fn new_cached_peering_record_store(
    inner: Rc<dyn PeeringRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringRecordStore> {
    Rc::new(CachedPeeringRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringRecord>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRecordQuery {
    pub category: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringRecordQuery {
    pub fn matches(&self, item: &PeeringRecord) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[PeeringRecord]) -> Vec<PeeringRecord> {
        let matched = collections::filter(items, |candidate: &PeeringRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_records(store: Rc<dyn PeeringRecordStore>, query: &PeeringRecordQuery) -> Vec<PeeringRecord> {
    query.apply(&store.list())
}
