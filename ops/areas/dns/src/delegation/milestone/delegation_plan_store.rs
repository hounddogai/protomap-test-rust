use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps delegation plans by code.
pub trait DelegationPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DelegationPlan>;
    fn save(&self, item: &DelegationPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DelegationPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelegationPlanStore {
    pub items: RefCell<BTreeMap<String, DelegationPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl DelegationPlanStore for MemoryDelegationPlanStore {
    fn find(&self, code: &str) -> Option<DelegationPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DelegationPlan) {
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

    fn list(&self) -> Vec<DelegationPlan> {
        let mut out: Vec<DelegationPlan> = Vec::new();
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

pub fn new_memory_delegation_plan_store() -> Rc<dyn DelegationPlanStore> {
    Rc::new(MemoryDelegationPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDelegationPlanStore {
    pub inner: Rc<dyn DelegationPlanStore>,
    pub cache: RefCell<cache::TtlCache<DelegationPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DelegationPlanStore for CachedDelegationPlanStore {
    fn find(&self, code: &str) -> Option<DelegationPlan> {
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

    fn save(&self, item: &DelegationPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DelegationPlan> {
        self.inner.list()
    }
}

pub fn new_cached_delegation_plan_store(
    inner: Rc<dyn DelegationPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DelegationPlanStore> {
    Rc::new(CachedDelegationPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DelegationPlan>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects delegation plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPlanQuery {
    pub title: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DelegationPlanQuery {
    pub fn matches(&self, item: &DelegationPlan) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DelegationPlan]) -> Vec<DelegationPlan> {
        let matched = collections::filter(items, |candidate: &DelegationPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_delegation_plans(store: Rc<dyn DelegationPlanStore>, query: &DelegationPlanQuery) -> Vec<DelegationPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DelegationPlanStore> = new_memory_delegation_plan_store();
        let item = new_delegation_plan("code-227", "title-600");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
