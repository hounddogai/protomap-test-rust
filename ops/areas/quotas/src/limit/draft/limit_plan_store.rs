use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps limit plans by code.
pub trait LimitPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LimitPlan>;
    fn save(&self, item: &LimitPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LimitPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLimitPlanStore {
    pub items: RefCell<BTreeMap<String, LimitPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl LimitPlanStore for MemoryLimitPlanStore {
    fn find(&self, code: &str) -> Option<LimitPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LimitPlan) {
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

    fn list(&self) -> Vec<LimitPlan> {
        let mut out: Vec<LimitPlan> = Vec::new();
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

pub fn new_memory_limit_plan_store() -> Rc<dyn LimitPlanStore> {
    Rc::new(MemoryLimitPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLimitPlanStore {
    pub inner: Rc<dyn LimitPlanStore>,
    pub cache: RefCell<cache::TtlCache<LimitPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LimitPlanStore for CachedLimitPlanStore {
    fn find(&self, code: &str) -> Option<LimitPlan> {
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

    fn save(&self, item: &LimitPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LimitPlan> {
        self.inner.list()
    }
}

pub fn new_cached_limit_plan_store(
    inner: Rc<dyn LimitPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LimitPlanStore> {
    Rc::new(CachedLimitPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LimitPlan>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects limit plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitPlanQuery {
    pub label: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LimitPlanQuery {
    pub fn matches(&self, item: &LimitPlan) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LimitPlan]) -> Vec<LimitPlan> {
        let matched = collections::filter(items, |candidate: &LimitPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_limit_plans(store: Rc<dyn LimitPlanStore>, query: &LimitPlanQuery) -> Vec<LimitPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LimitPlanStore> = new_memory_limit_plan_store();
        let item = new_limit_plan("code-119", "label-547");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
