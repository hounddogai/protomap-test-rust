use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps delegation metrics by code.
pub trait DelegationMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DelegationMetric>;
    fn save(&self, item: &DelegationMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DelegationMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelegationMetricStore {
    pub items: RefCell<BTreeMap<String, DelegationMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl DelegationMetricStore for MemoryDelegationMetricStore {
    fn find(&self, code: &str) -> Option<DelegationMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DelegationMetric) {
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

    fn list(&self) -> Vec<DelegationMetric> {
        let mut out: Vec<DelegationMetric> = Vec::new();
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

pub fn new_memory_delegation_metric_store() -> Rc<dyn DelegationMetricStore> {
    Rc::new(MemoryDelegationMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDelegationMetricStore {
    pub inner: Rc<dyn DelegationMetricStore>,
    pub cache: RefCell<cache::TtlCache<DelegationMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DelegationMetricStore for CachedDelegationMetricStore {
    fn find(&self, code: &str) -> Option<DelegationMetric> {
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

    fn save(&self, item: &DelegationMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DelegationMetric> {
        self.inner.list()
    }
}

pub fn new_cached_delegation_metric_store(
    inner: Rc<dyn DelegationMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DelegationMetricStore> {
    Rc::new(CachedDelegationMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DelegationMetric>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects delegation metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMetricQuery {
    pub scope: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DelegationMetricQuery {
    pub fn matches(&self, item: &DelegationMetric) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[DelegationMetric]) -> Vec<DelegationMetric> {
        let matched = collections::filter(items, |candidate: &DelegationMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_delegation_metrics(
    store: Rc<dyn DelegationMetricStore>,
    query: &DelegationMetricQuery,
) -> Vec<DelegationMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DelegationMetricStore> = new_memory_delegation_metric_store();
        let item = new_delegation_metric("code-178", "name-1");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
