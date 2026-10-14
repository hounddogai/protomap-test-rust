use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy limit plans by code.
pub trait LegacyLimitPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyLimitPlan>;
    fn save(&self, item: &LegacyLimitPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyLimitPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyLimitPlanStore {
    pub items: RefCell<BTreeMap<String, LegacyLimitPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyLimitPlanStore for MemoryLegacyLimitPlanStore {
    fn find(&self, code: &str) -> Option<LegacyLimitPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyLimitPlan) {
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

    fn list(&self) -> Vec<LegacyLimitPlan> {
        let mut out: Vec<LegacyLimitPlan> = Vec::new();
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

pub fn new_memory_legacy_limit_plan_store() -> Rc<dyn LegacyLimitPlanStore> {
    Rc::new(MemoryLegacyLimitPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyLimitPlanStore {
    pub inner: Rc<dyn LegacyLimitPlanStore>,
    pub cache: RefCell<cache::TtlCache<LegacyLimitPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyLimitPlanStore for CachedLegacyLimitPlanStore {
    fn find(&self, code: &str) -> Option<LegacyLimitPlan> {
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

    fn save(&self, item: &LegacyLimitPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyLimitPlan> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_limit_plan_store(
    inner: Rc<dyn LegacyLimitPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyLimitPlanStore> {
    Rc::new(CachedLegacyLimitPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyLimitPlan>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy limit plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlanQuery {
    pub name: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyLimitPlanQuery {
    pub fn matches(&self, item: &LegacyLimitPlan) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyLimitPlan]) -> Vec<LegacyLimitPlan> {
        let matched = collections::filter(items, |candidate: &LegacyLimitPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_limit_plans(
    store: Rc<dyn LegacyLimitPlanStore>,
    query: &LegacyLimitPlanQuery,
) -> Vec<LegacyLimitPlan> {
    query.apply(&store.list())
}
