use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback region pin plans by code.
pub trait FallbackRegionPinPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackRegionPinPlan>;
    fn save(&self, item: &FallbackRegionPinPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackRegionPinPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackRegionPinPlanStore {
    pub items: RefCell<BTreeMap<String, FallbackRegionPinPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackRegionPinPlanStore for MemoryFallbackRegionPinPlanStore {
    fn find(&self, code: &str) -> Option<FallbackRegionPinPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackRegionPinPlan) {
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

    fn list(&self) -> Vec<FallbackRegionPinPlan> {
        let mut out: Vec<FallbackRegionPinPlan> = Vec::new();
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

pub fn new_memory_fallback_region_pin_plan_store() -> Rc<dyn FallbackRegionPinPlanStore> {
    Rc::new(MemoryFallbackRegionPinPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackRegionPinPlanStore {
    pub inner: Rc<dyn FallbackRegionPinPlanStore>,
    pub cache: RefCell<cache::TtlCache<FallbackRegionPinPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackRegionPinPlanStore for CachedFallbackRegionPinPlanStore {
    fn find(&self, code: &str) -> Option<FallbackRegionPinPlan> {
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

    fn save(&self, item: &FallbackRegionPinPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackRegionPinPlan> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_region_pin_plan_store(
    inner: Rc<dyn FallbackRegionPinPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackRegionPinPlanStore> {
    Rc::new(CachedFallbackRegionPinPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackRegionPinPlan>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback region pin plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinPlanQuery {
    pub label: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackRegionPinPlanQuery {
    pub fn matches(&self, item: &FallbackRegionPinPlan) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackRegionPinPlan]) -> Vec<FallbackRegionPinPlan> {
        let matched = collections::filter(items, |candidate: &FallbackRegionPinPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_region_pin_plans(
    store: Rc<dyn FallbackRegionPinPlanStore>,
    query: &FallbackRegionPinPlanQuery,
) -> Vec<FallbackRegionPinPlan> {
    query.apply(&store.list())
}
