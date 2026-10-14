use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending limit plans by code.
pub trait PendingLimitPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingLimitPlan>;
    fn save(&self, item: &PendingLimitPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingLimitPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingLimitPlanStore {
    pub items: RefCell<BTreeMap<String, PendingLimitPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingLimitPlanStore for MemoryPendingLimitPlanStore {
    fn find(&self, code: &str) -> Option<PendingLimitPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingLimitPlan) {
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

    fn list(&self) -> Vec<PendingLimitPlan> {
        let mut out: Vec<PendingLimitPlan> = Vec::new();
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

pub fn new_memory_pending_limit_plan_store() -> Rc<dyn PendingLimitPlanStore> {
    Rc::new(MemoryPendingLimitPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingLimitPlanStore {
    pub inner: Rc<dyn PendingLimitPlanStore>,
    pub cache: RefCell<cache::TtlCache<PendingLimitPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingLimitPlanStore for CachedPendingLimitPlanStore {
    fn find(&self, code: &str) -> Option<PendingLimitPlan> {
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

    fn save(&self, item: &PendingLimitPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingLimitPlan> {
        self.inner.list()
    }
}

pub fn new_cached_pending_limit_plan_store(
    inner: Rc<dyn PendingLimitPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingLimitPlanStore> {
    Rc::new(CachedPendingLimitPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingLimitPlan>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending limit plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPlanQuery {
    pub scope: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingLimitPlanQuery {
    pub fn matches(&self, item: &PendingLimitPlan) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[PendingLimitPlan]) -> Vec<PendingLimitPlan> {
        let matched = collections::filter(items, |candidate: &PendingLimitPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_limit_plans(
    store: Rc<dyn PendingLimitPlanStore>,
    query: &PendingLimitPlanQuery,
) -> Vec<PendingLimitPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingLimitPlanStore> = new_memory_pending_limit_plan_store();
        let item = new_pending_limit_plan("code-753", "title-125");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
