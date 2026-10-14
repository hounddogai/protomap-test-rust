use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling ttl policy estimates by code.
pub trait RollingTtlPolicyEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingTtlPolicyEstimate>;
    fn save(&self, item: &RollingTtlPolicyEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingTtlPolicyEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingTtlPolicyEstimateStore {
    pub items: RefCell<BTreeMap<String, RollingTtlPolicyEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingTtlPolicyEstimateStore for MemoryRollingTtlPolicyEstimateStore {
    fn find(&self, code: &str) -> Option<RollingTtlPolicyEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingTtlPolicyEstimate) {
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

    fn list(&self) -> Vec<RollingTtlPolicyEstimate> {
        let mut out: Vec<RollingTtlPolicyEstimate> = Vec::new();
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

pub fn new_memory_rolling_ttl_policy_estimate_store() -> Rc<dyn RollingTtlPolicyEstimateStore> {
    Rc::new(MemoryRollingTtlPolicyEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingTtlPolicyEstimateStore {
    pub inner: Rc<dyn RollingTtlPolicyEstimateStore>,
    pub cache: RefCell<cache::TtlCache<RollingTtlPolicyEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingTtlPolicyEstimateStore for CachedRollingTtlPolicyEstimateStore {
    fn find(&self, code: &str) -> Option<RollingTtlPolicyEstimate> {
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

    fn save(&self, item: &RollingTtlPolicyEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingTtlPolicyEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_ttl_policy_estimate_store(
    inner: Rc<dyn RollingTtlPolicyEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingTtlPolicyEstimateStore> {
    Rc::new(CachedRollingTtlPolicyEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingTtlPolicyEstimate>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling ttl policy estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyEstimateQuery {
    pub namespace: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingTtlPolicyEstimateQuery {
    pub fn matches(&self, item: &RollingTtlPolicyEstimate) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[RollingTtlPolicyEstimate]) -> Vec<RollingTtlPolicyEstimate> {
        let matched = collections::filter(items, |candidate: &RollingTtlPolicyEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_ttl_policy_estimates(
    store: Rc<dyn RollingTtlPolicyEstimateStore>,
    query: &RollingTtlPolicyEstimateQuery,
) -> Vec<RollingTtlPolicyEstimate> {
    query.apply(&store.list())
}
