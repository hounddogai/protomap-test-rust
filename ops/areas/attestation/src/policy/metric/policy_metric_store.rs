use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps policy metrics by code.
pub trait PolicyMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PolicyMetric>;
    fn save(&self, item: &PolicyMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PolicyMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPolicyMetricStore {
    pub items: RefCell<BTreeMap<String, PolicyMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl PolicyMetricStore for MemoryPolicyMetricStore {
    fn find(&self, code: &str) -> Option<PolicyMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PolicyMetric) {
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

    fn list(&self) -> Vec<PolicyMetric> {
        let mut out: Vec<PolicyMetric> = Vec::new();
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

pub fn new_memory_policy_metric_store() -> Rc<dyn PolicyMetricStore> {
    Rc::new(MemoryPolicyMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPolicyMetricStore {
    pub inner: Rc<dyn PolicyMetricStore>,
    pub cache: RefCell<cache::TtlCache<PolicyMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PolicyMetricStore for CachedPolicyMetricStore {
    fn find(&self, code: &str) -> Option<PolicyMetric> {
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

    fn save(&self, item: &PolicyMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PolicyMetric> {
        self.inner.list()
    }
}

pub fn new_cached_policy_metric_store(
    inner: Rc<dyn PolicyMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PolicyMetricStore> {
    Rc::new(CachedPolicyMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PolicyMetric>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects policy metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMetricQuery {
    pub cluster: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PolicyMetricQuery {
    pub fn matches(&self, item: &PolicyMetric) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[PolicyMetric]) -> Vec<PolicyMetric> {
        let matched = collections::filter(items, |candidate: &PolicyMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_policy_metrics(store: Rc<dyn PolicyMetricStore>, query: &PolicyMetricQuery) -> Vec<PolicyMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PolicyMetricStore> = new_memory_policy_metric_store();
        let item = new_policy_metric("code-600", "label-155");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
