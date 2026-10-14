use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps canary plans by code.
pub trait CanaryPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CanaryPlan>;
    fn save(&self, item: &CanaryPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CanaryPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCanaryPlanStore {
    pub items: RefCell<BTreeMap<String, CanaryPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl CanaryPlanStore for MemoryCanaryPlanStore {
    fn find(&self, code: &str) -> Option<CanaryPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CanaryPlan) {
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

    fn list(&self) -> Vec<CanaryPlan> {
        let mut out: Vec<CanaryPlan> = Vec::new();
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

pub fn new_memory_canary_plan_store() -> Rc<dyn CanaryPlanStore> {
    Rc::new(MemoryCanaryPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCanaryPlanStore {
    pub inner: Rc<dyn CanaryPlanStore>,
    pub cache: RefCell<cache::TtlCache<CanaryPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CanaryPlanStore for CachedCanaryPlanStore {
    fn find(&self, code: &str) -> Option<CanaryPlan> {
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

    fn save(&self, item: &CanaryPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CanaryPlan> {
        self.inner.list()
    }
}

pub fn new_cached_canary_plan_store(
    inner: Rc<dyn CanaryPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CanaryPlanStore> {
    Rc::new(CachedCanaryPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CanaryPlan>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects canary plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPlanQuery {
    pub name: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CanaryPlanQuery {
    pub fn matches(&self, item: &CanaryPlan) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CanaryPlan]) -> Vec<CanaryPlan> {
        let matched = collections::filter(items, |candidate: &CanaryPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_canary_plans(store: Rc<dyn CanaryPlanStore>, query: &CanaryPlanQuery) -> Vec<CanaryPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CanaryPlanStore> = new_memory_canary_plan_store();
        let item = new_canary_plan("code-439", "name-919");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
