use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global policy plans by code.
pub trait GlobalPolicyPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalPolicyPlan>;
    fn save(&self, item: &GlobalPolicyPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalPolicyPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalPolicyPlanStore {
    pub items: RefCell<BTreeMap<String, GlobalPolicyPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalPolicyPlanStore for MemoryGlobalPolicyPlanStore {
    fn find(&self, code: &str) -> Option<GlobalPolicyPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalPolicyPlan) {
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

    fn list(&self) -> Vec<GlobalPolicyPlan> {
        let mut out: Vec<GlobalPolicyPlan> = Vec::new();
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

pub fn new_memory_global_policy_plan_store() -> Rc<dyn GlobalPolicyPlanStore> {
    Rc::new(MemoryGlobalPolicyPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalPolicyPlanStore {
    pub inner: Rc<dyn GlobalPolicyPlanStore>,
    pub cache: RefCell<cache::TtlCache<GlobalPolicyPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalPolicyPlanStore for CachedGlobalPolicyPlanStore {
    fn find(&self, code: &str) -> Option<GlobalPolicyPlan> {
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

    fn save(&self, item: &GlobalPolicyPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalPolicyPlan> {
        self.inner.list()
    }
}

pub fn new_cached_global_policy_plan_store(
    inner: Rc<dyn GlobalPolicyPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalPolicyPlanStore> {
    Rc::new(CachedGlobalPolicyPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalPolicyPlan>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global policy plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlanQuery {
    pub cluster: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalPolicyPlanQuery {
    pub fn matches(&self, item: &GlobalPolicyPlan) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalPolicyPlan]) -> Vec<GlobalPolicyPlan> {
        let matched = collections::filter(items, |candidate: &GlobalPolicyPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_policy_plans(
    store: Rc<dyn GlobalPolicyPlanStore>,
    query: &GlobalPolicyPlanQuery,
) -> Vec<GlobalPolicyPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalPolicyPlanStore> = new_memory_global_policy_plan_store();
        let item = new_global_policy_plan("code-878", "name-442");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
