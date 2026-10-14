use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement plans by code.
pub trait MeasurementPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementPlan>;
    fn save(&self, item: &MeasurementPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementPlanStore {
    pub items: RefCell<BTreeMap<String, MeasurementPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementPlanStore for MemoryMeasurementPlanStore {
    fn find(&self, code: &str) -> Option<MeasurementPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementPlan) {
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

    fn list(&self) -> Vec<MeasurementPlan> {
        let mut out: Vec<MeasurementPlan> = Vec::new();
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

pub fn new_memory_measurement_plan_store() -> Rc<dyn MeasurementPlanStore> {
    Rc::new(MemoryMeasurementPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementPlanStore {
    pub inner: Rc<dyn MeasurementPlanStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementPlanStore for CachedMeasurementPlanStore {
    fn find(&self, code: &str) -> Option<MeasurementPlan> {
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

    fn save(&self, item: &MeasurementPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementPlan> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_plan_store(
    inner: Rc<dyn MeasurementPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementPlanStore> {
    Rc::new(CachedMeasurementPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementPlan>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPlanQuery {
    pub source: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementPlanQuery {
    pub fn matches(&self, item: &MeasurementPlan) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementPlan]) -> Vec<MeasurementPlan> {
        let matched = collections::filter(items, |candidate: &MeasurementPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_plans(
    store: Rc<dyn MeasurementPlanStore>,
    query: &MeasurementPlanQuery,
) -> Vec<MeasurementPlan> {
    query.apply(&store.list())
}
