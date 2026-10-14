use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan records by code.
pub trait PlanRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanRecord>;
    fn save(&self, item: &PlanRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanRecordStore {
    pub items: RefCell<BTreeMap<String, PlanRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanRecordStore for MemoryPlanRecordStore {
    fn find(&self, code: &str) -> Option<PlanRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanRecord) {
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

    fn list(&self) -> Vec<PlanRecord> {
        let mut out: Vec<PlanRecord> = Vec::new();
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

pub fn new_memory_plan_record_store() -> Rc<dyn PlanRecordStore> {
    Rc::new(MemoryPlanRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanRecordStore {
    pub inner: Rc<dyn PlanRecordStore>,
    pub cache: RefCell<cache::TtlCache<PlanRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanRecordStore for CachedPlanRecordStore {
    fn find(&self, code: &str) -> Option<PlanRecord> {
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

    fn save(&self, item: &PlanRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanRecord> {
        self.inner.list()
    }
}

pub fn new_cached_plan_record_store(
    inner: Rc<dyn PlanRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanRecordStore> {
    Rc::new(CachedPlanRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanRecord>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRecordQuery {
    pub title: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanRecordQuery {
    pub fn matches(&self, item: &PlanRecord) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[PlanRecord]) -> Vec<PlanRecord> {
        let matched = collections::filter(items, |candidate: &PlanRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_records(store: Rc<dyn PlanRecordStore>, query: &PlanRecordQuery) -> Vec<PlanRecord> {
    query.apply(&store.list())
}
