use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity plans by code.
pub trait IdentityPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityPlan>;
    fn save(&self, item: &IdentityPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityPlanStore {
    pub items: RefCell<BTreeMap<String, IdentityPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityPlanStore for MemoryIdentityPlanStore {
    fn find(&self, code: &str) -> Option<IdentityPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityPlan) {
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

    fn list(&self) -> Vec<IdentityPlan> {
        let mut out: Vec<IdentityPlan> = Vec::new();
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

pub fn new_memory_identity_plan_store() -> Rc<dyn IdentityPlanStore> {
    Rc::new(MemoryIdentityPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityPlanStore {
    pub inner: Rc<dyn IdentityPlanStore>,
    pub cache: RefCell<cache::TtlCache<IdentityPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityPlanStore for CachedIdentityPlanStore {
    fn find(&self, code: &str) -> Option<IdentityPlan> {
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

    fn save(&self, item: &IdentityPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityPlan> {
        self.inner.list()
    }
}

pub fn new_cached_identity_plan_store(
    inner: Rc<dyn IdentityPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityPlanStore> {
    Rc::new(CachedIdentityPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityPlan>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPlanQuery {
    pub cluster: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityPlanQuery {
    pub fn matches(&self, item: &IdentityPlan) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IdentityPlan]) -> Vec<IdentityPlan> {
        let matched = collections::filter(items, |candidate: &IdentityPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_plans(store: Rc<dyn IdentityPlanStore>, query: &IdentityPlanQuery) -> Vec<IdentityPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IdentityPlanStore> = new_memory_identity_plan_store();
        let item = new_identity_plan("code-808", "title-492");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
