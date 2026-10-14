use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval plans by code.
pub trait ApprovalPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalPlan>;
    fn save(&self, item: &ApprovalPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalPlanStore {
    pub items: RefCell<BTreeMap<String, ApprovalPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalPlanStore for MemoryApprovalPlanStore {
    fn find(&self, code: &str) -> Option<ApprovalPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalPlan) {
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

    fn list(&self) -> Vec<ApprovalPlan> {
        let mut out: Vec<ApprovalPlan> = Vec::new();
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

pub fn new_memory_approval_plan_store() -> Rc<dyn ApprovalPlanStore> {
    Rc::new(MemoryApprovalPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalPlanStore {
    pub inner: Rc<dyn ApprovalPlanStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalPlanStore for CachedApprovalPlanStore {
    fn find(&self, code: &str) -> Option<ApprovalPlan> {
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

    fn save(&self, item: &ApprovalPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalPlan> {
        self.inner.list()
    }
}

pub fn new_cached_approval_plan_store(
    inner: Rc<dyn ApprovalPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalPlanStore> {
    Rc::new(CachedApprovalPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalPlan>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPlanQuery {
    pub country: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalPlanQuery {
    pub fn matches(&self, item: &ApprovalPlan) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ApprovalPlan]) -> Vec<ApprovalPlan> {
        let matched = collections::filter(items, |candidate: &ApprovalPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_plans(store: Rc<dyn ApprovalPlanStore>, query: &ApprovalPlanQuery) -> Vec<ApprovalPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ApprovalPlanStore> = new_memory_approval_plan_store();
        let item = new_approval_plan("code-327", "name-585");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
