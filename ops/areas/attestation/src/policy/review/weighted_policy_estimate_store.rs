use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted policy estimates by code.
pub trait WeightedPolicyEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedPolicyEstimate>;
    fn save(&self, item: &WeightedPolicyEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedPolicyEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedPolicyEstimateStore {
    pub items: RefCell<BTreeMap<String, WeightedPolicyEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedPolicyEstimateStore for MemoryWeightedPolicyEstimateStore {
    fn find(&self, code: &str) -> Option<WeightedPolicyEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedPolicyEstimate) {
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

    fn list(&self) -> Vec<WeightedPolicyEstimate> {
        let mut out: Vec<WeightedPolicyEstimate> = Vec::new();
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

pub fn new_memory_weighted_policy_estimate_store() -> Rc<dyn WeightedPolicyEstimateStore> {
    Rc::new(MemoryWeightedPolicyEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedPolicyEstimateStore {
    pub inner: Rc<dyn WeightedPolicyEstimateStore>,
    pub cache: RefCell<cache::TtlCache<WeightedPolicyEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedPolicyEstimateStore for CachedWeightedPolicyEstimateStore {
    fn find(&self, code: &str) -> Option<WeightedPolicyEstimate> {
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

    fn save(&self, item: &WeightedPolicyEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedPolicyEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_policy_estimate_store(
    inner: Rc<dyn WeightedPolicyEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedPolicyEstimateStore> {
    Rc::new(CachedWeightedPolicyEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedPolicyEstimate>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted policy estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyEstimateQuery {
    pub label: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedPolicyEstimateQuery {
    pub fn matches(&self, item: &WeightedPolicyEstimate) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedPolicyEstimate]) -> Vec<WeightedPolicyEstimate> {
        let matched = collections::filter(items, |candidate: &WeightedPolicyEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_policy_estimates(
    store: Rc<dyn WeightedPolicyEstimateStore>,
    query: &WeightedPolicyEstimateQuery,
) -> Vec<WeightedPolicyEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedPolicyEstimateStore> = new_memory_weighted_policy_estimate_store();
        let item = new_weighted_policy_estimate("code-442", "label-498");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
