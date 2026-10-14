use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps policy estimates by code.
pub trait PolicyEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PolicyEstimate>;
    fn save(&self, item: &PolicyEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PolicyEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPolicyEstimateStore {
    pub items: RefCell<BTreeMap<String, PolicyEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl PolicyEstimateStore for MemoryPolicyEstimateStore {
    fn find(&self, code: &str) -> Option<PolicyEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PolicyEstimate) {
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

    fn list(&self) -> Vec<PolicyEstimate> {
        let mut out: Vec<PolicyEstimate> = Vec::new();
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

pub fn new_memory_policy_estimate_store() -> Rc<dyn PolicyEstimateStore> {
    Rc::new(MemoryPolicyEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPolicyEstimateStore {
    pub inner: Rc<dyn PolicyEstimateStore>,
    pub cache: RefCell<cache::TtlCache<PolicyEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PolicyEstimateStore for CachedPolicyEstimateStore {
    fn find(&self, code: &str) -> Option<PolicyEstimate> {
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

    fn save(&self, item: &PolicyEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PolicyEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_policy_estimate_store(
    inner: Rc<dyn PolicyEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PolicyEstimateStore> {
    Rc::new(CachedPolicyEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PolicyEstimate>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects policy estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyEstimateQuery {
    pub segment: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PolicyEstimateQuery {
    pub fn matches(&self, item: &PolicyEstimate) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PolicyEstimate]) -> Vec<PolicyEstimate> {
        let matched = collections::filter(items, |candidate: &PolicyEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_policy_estimates(store: Rc<dyn PolicyEstimateStore>, query: &PolicyEstimateQuery) -> Vec<PolicyEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PolicyEstimateStore> = new_memory_policy_estimate_store();
        let item = new_policy_estimate("code-911", "label-139");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
