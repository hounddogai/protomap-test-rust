use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted timeline plans by code.
pub trait WeightedTimelinePlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedTimelinePlan>;
    fn save(&self, item: &WeightedTimelinePlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedTimelinePlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedTimelinePlanStore {
    pub items: RefCell<BTreeMap<String, WeightedTimelinePlan>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedTimelinePlanStore for MemoryWeightedTimelinePlanStore {
    fn find(&self, code: &str) -> Option<WeightedTimelinePlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedTimelinePlan) {
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

    fn list(&self) -> Vec<WeightedTimelinePlan> {
        let mut out: Vec<WeightedTimelinePlan> = Vec::new();
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

pub fn new_memory_weighted_timeline_plan_store() -> Rc<dyn WeightedTimelinePlanStore> {
    Rc::new(MemoryWeightedTimelinePlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedTimelinePlanStore {
    pub inner: Rc<dyn WeightedTimelinePlanStore>,
    pub cache: RefCell<cache::TtlCache<WeightedTimelinePlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedTimelinePlanStore for CachedWeightedTimelinePlanStore {
    fn find(&self, code: &str) -> Option<WeightedTimelinePlan> {
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

    fn save(&self, item: &WeightedTimelinePlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedTimelinePlan> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_timeline_plan_store(
    inner: Rc<dyn WeightedTimelinePlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedTimelinePlanStore> {
    Rc::new(CachedWeightedTimelinePlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedTimelinePlan>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted timeline plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelinePlanQuery {
    pub label: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedTimelinePlanQuery {
    pub fn matches(&self, item: &WeightedTimelinePlan) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[WeightedTimelinePlan]) -> Vec<WeightedTimelinePlan> {
        let matched = collections::filter(items, |candidate: &WeightedTimelinePlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_timeline_plans(
    store: Rc<dyn WeightedTimelinePlanStore>,
    query: &WeightedTimelinePlanQuery,
) -> Vec<WeightedTimelinePlan> {
    query.apply(&store.list())
}
