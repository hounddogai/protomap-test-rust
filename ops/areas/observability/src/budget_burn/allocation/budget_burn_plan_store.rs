use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps budget burn plans by code.
pub trait BudgetBurnPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BudgetBurnPlan>;
    fn save(&self, item: &BudgetBurnPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BudgetBurnPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBudgetBurnPlanStore {
    pub items: RefCell<BTreeMap<String, BudgetBurnPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl BudgetBurnPlanStore for MemoryBudgetBurnPlanStore {
    fn find(&self, code: &str) -> Option<BudgetBurnPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BudgetBurnPlan) {
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

    fn list(&self) -> Vec<BudgetBurnPlan> {
        let mut out: Vec<BudgetBurnPlan> = Vec::new();
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

pub fn new_memory_budget_burn_plan_store() -> Rc<dyn BudgetBurnPlanStore> {
    Rc::new(MemoryBudgetBurnPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBudgetBurnPlanStore {
    pub inner: Rc<dyn BudgetBurnPlanStore>,
    pub cache: RefCell<cache::TtlCache<BudgetBurnPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BudgetBurnPlanStore for CachedBudgetBurnPlanStore {
    fn find(&self, code: &str) -> Option<BudgetBurnPlan> {
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

    fn save(&self, item: &BudgetBurnPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BudgetBurnPlan> {
        self.inner.list()
    }
}

pub fn new_cached_budget_burn_plan_store(
    inner: Rc<dyn BudgetBurnPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BudgetBurnPlanStore> {
    Rc::new(CachedBudgetBurnPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BudgetBurnPlan>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects budget burn plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPlanQuery {
    pub cluster: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BudgetBurnPlanQuery {
    pub fn matches(&self, item: &BudgetBurnPlan) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[BudgetBurnPlan]) -> Vec<BudgetBurnPlan> {
        let matched = collections::filter(items, |candidate: &BudgetBurnPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_budget_burn_plans(store: Rc<dyn BudgetBurnPlanStore>, query: &BudgetBurnPlanQuery) -> Vec<BudgetBurnPlan> {
    query.apply(&store.list())
}
