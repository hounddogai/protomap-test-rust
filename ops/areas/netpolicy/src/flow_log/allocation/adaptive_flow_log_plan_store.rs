use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive flow log plans by code.
pub trait AdaptiveFlowLogPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveFlowLogPlan>;
    fn save(&self, item: &AdaptiveFlowLogPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveFlowLogPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveFlowLogPlanStore {
    pub items: RefCell<BTreeMap<String, AdaptiveFlowLogPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveFlowLogPlanStore for MemoryAdaptiveFlowLogPlanStore {
    fn find(&self, code: &str) -> Option<AdaptiveFlowLogPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveFlowLogPlan) {
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

    fn list(&self) -> Vec<AdaptiveFlowLogPlan> {
        let mut out: Vec<AdaptiveFlowLogPlan> = Vec::new();
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

pub fn new_memory_adaptive_flow_log_plan_store() -> Rc<dyn AdaptiveFlowLogPlanStore> {
    Rc::new(MemoryAdaptiveFlowLogPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveFlowLogPlanStore {
    pub inner: Rc<dyn AdaptiveFlowLogPlanStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveFlowLogPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveFlowLogPlanStore for CachedAdaptiveFlowLogPlanStore {
    fn find(&self, code: &str) -> Option<AdaptiveFlowLogPlan> {
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

    fn save(&self, item: &AdaptiveFlowLogPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveFlowLogPlan> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_flow_log_plan_store(
    inner: Rc<dyn AdaptiveFlowLogPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveFlowLogPlanStore> {
    Rc::new(CachedAdaptiveFlowLogPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveFlowLogPlan>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive flow log plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogPlanQuery {
    pub name: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveFlowLogPlanQuery {
    pub fn matches(&self, item: &AdaptiveFlowLogPlan) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[AdaptiveFlowLogPlan]) -> Vec<AdaptiveFlowLogPlan> {
        let matched = collections::filter(items, |candidate: &AdaptiveFlowLogPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_flow_log_plans(
    store: Rc<dyn AdaptiveFlowLogPlanStore>,
    query: &AdaptiveFlowLogPlanQuery,
) -> Vec<AdaptiveFlowLogPlan> {
    query.apply(&store.list())
}
