use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling region pin plans by code.
pub trait RollingRegionPinPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingRegionPinPlan>;
    fn save(&self, item: &RollingRegionPinPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingRegionPinPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingRegionPinPlanStore {
    pub items: RefCell<BTreeMap<String, RollingRegionPinPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingRegionPinPlanStore for MemoryRollingRegionPinPlanStore {
    fn find(&self, code: &str) -> Option<RollingRegionPinPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingRegionPinPlan) {
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

    fn list(&self) -> Vec<RollingRegionPinPlan> {
        let mut out: Vec<RollingRegionPinPlan> = Vec::new();
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

pub fn new_memory_rolling_region_pin_plan_store() -> Rc<dyn RollingRegionPinPlanStore> {
    Rc::new(MemoryRollingRegionPinPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingRegionPinPlanStore {
    pub inner: Rc<dyn RollingRegionPinPlanStore>,
    pub cache: RefCell<cache::TtlCache<RollingRegionPinPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingRegionPinPlanStore for CachedRollingRegionPinPlanStore {
    fn find(&self, code: &str) -> Option<RollingRegionPinPlan> {
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

    fn save(&self, item: &RollingRegionPinPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingRegionPinPlan> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_region_pin_plan_store(
    inner: Rc<dyn RollingRegionPinPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingRegionPinPlanStore> {
    Rc::new(CachedRollingRegionPinPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingRegionPinPlan>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling region pin plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlanQuery {
    pub source: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingRegionPinPlanQuery {
    pub fn matches(&self, item: &RollingRegionPinPlan) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingRegionPinPlan]) -> Vec<RollingRegionPinPlan> {
        let matched = collections::filter(items, |candidate: &RollingRegionPinPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_region_pin_plans(
    store: Rc<dyn RollingRegionPinPlanStore>,
    query: &RollingRegionPinPlanQuery,
) -> Vec<RollingRegionPinPlan> {
    query.apply(&store.list())
}
