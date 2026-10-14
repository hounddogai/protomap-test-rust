use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal budget burn plans by code.
pub trait SeasonalBudgetBurnPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalBudgetBurnPlan>;
    fn save(&self, item: &SeasonalBudgetBurnPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalBudgetBurnPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalBudgetBurnPlanStore {
    pub items: RefCell<BTreeMap<String, SeasonalBudgetBurnPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalBudgetBurnPlanStore for MemorySeasonalBudgetBurnPlanStore {
    fn find(&self, code: &str) -> Option<SeasonalBudgetBurnPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalBudgetBurnPlan) {
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

    fn list(&self) -> Vec<SeasonalBudgetBurnPlan> {
        let mut out: Vec<SeasonalBudgetBurnPlan> = Vec::new();
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

pub fn new_memory_seasonal_budget_burn_plan_store() -> Rc<dyn SeasonalBudgetBurnPlanStore> {
    Rc::new(MemorySeasonalBudgetBurnPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalBudgetBurnPlanStore {
    pub inner: Rc<dyn SeasonalBudgetBurnPlanStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalBudgetBurnPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalBudgetBurnPlanStore for CachedSeasonalBudgetBurnPlanStore {
    fn find(&self, code: &str) -> Option<SeasonalBudgetBurnPlan> {
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

    fn save(&self, item: &SeasonalBudgetBurnPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalBudgetBurnPlan> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_budget_burn_plan_store(
    inner: Rc<dyn SeasonalBudgetBurnPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalBudgetBurnPlanStore> {
    Rc::new(CachedSeasonalBudgetBurnPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalBudgetBurnPlan>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal budget burn plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnPlanQuery {
    pub segment: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalBudgetBurnPlanQuery {
    pub fn matches(&self, item: &SeasonalBudgetBurnPlan) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalBudgetBurnPlan]) -> Vec<SeasonalBudgetBurnPlan> {
        let matched = collections::filter(items, |candidate: &SeasonalBudgetBurnPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_budget_burn_plans(
    store: Rc<dyn SeasonalBudgetBurnPlanStore>,
    query: &SeasonalBudgetBurnPlanQuery,
) -> Vec<SeasonalBudgetBurnPlan> {
    query.apply(&store.list())
}
