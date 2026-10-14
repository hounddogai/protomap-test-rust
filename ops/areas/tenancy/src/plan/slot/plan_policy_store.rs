use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan policies by code.
pub trait PlanPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanPolicy>;
    fn save(&self, item: &PlanPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanPolicyStore {
    pub items: RefCell<BTreeMap<String, PlanPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanPolicyStore for MemoryPlanPolicyStore {
    fn find(&self, code: &str) -> Option<PlanPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanPolicy) {
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

    fn list(&self) -> Vec<PlanPolicy> {
        let mut out: Vec<PlanPolicy> = Vec::new();
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

pub fn new_memory_plan_policy_store() -> Rc<dyn PlanPolicyStore> {
    Rc::new(MemoryPlanPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanPolicyStore {
    pub inner: Rc<dyn PlanPolicyStore>,
    pub cache: RefCell<cache::TtlCache<PlanPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanPolicyStore for CachedPlanPolicyStore {
    fn find(&self, code: &str) -> Option<PlanPolicy> {
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

    fn save(&self, item: &PlanPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_plan_policy_store(
    inner: Rc<dyn PlanPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanPolicyStore> {
    Rc::new(CachedPlanPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanPolicy>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanPolicyQuery {
    pub name: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanPolicyQuery {
    pub fn matches(&self, item: &PlanPolicy) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[PlanPolicy]) -> Vec<PlanPolicy> {
        let matched = collections::filter(items, |candidate: &PlanPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_policies(store: Rc<dyn PlanPolicyStore>, query: &PlanPolicyQuery) -> Vec<PlanPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PlanPolicyStore> = new_memory_plan_policy_store();
        let item = new_plan_policy("code-951", "name-360");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
