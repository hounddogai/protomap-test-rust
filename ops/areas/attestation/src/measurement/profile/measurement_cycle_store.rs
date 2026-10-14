use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement cycles by code.
pub trait MeasurementCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementCycle>;
    fn save(&self, item: &MeasurementCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementCycleStore {
    pub items: RefCell<BTreeMap<String, MeasurementCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementCycleStore for MemoryMeasurementCycleStore {
    fn find(&self, code: &str) -> Option<MeasurementCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementCycle) {
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

    fn list(&self) -> Vec<MeasurementCycle> {
        let mut out: Vec<MeasurementCycle> = Vec::new();
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

pub fn new_memory_measurement_cycle_store() -> Rc<dyn MeasurementCycleStore> {
    Rc::new(MemoryMeasurementCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementCycleStore {
    pub inner: Rc<dyn MeasurementCycleStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementCycleStore for CachedMeasurementCycleStore {
    fn find(&self, code: &str) -> Option<MeasurementCycle> {
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

    fn save(&self, item: &MeasurementCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementCycle> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_cycle_store(
    inner: Rc<dyn MeasurementCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementCycleStore> {
    Rc::new(CachedMeasurementCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementCycle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementCycleQuery {
    pub category: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementCycleQuery {
    pub fn matches(&self, item: &MeasurementCycle) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementCycle]) -> Vec<MeasurementCycle> {
        let matched = collections::filter(items, |candidate: &MeasurementCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_cycles(
    store: Rc<dyn MeasurementCycleStore>,
    query: &MeasurementCycleQuery,
) -> Vec<MeasurementCycle> {
    query.apply(&store.list())
}
