use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollout estimates by code.
pub trait RolloutEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RolloutEstimate>;
    fn save(&self, item: &RolloutEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RolloutEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRolloutEstimateStore {
    pub items: RefCell<BTreeMap<String, RolloutEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl RolloutEstimateStore for MemoryRolloutEstimateStore {
    fn find(&self, code: &str) -> Option<RolloutEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RolloutEstimate) {
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

    fn list(&self) -> Vec<RolloutEstimate> {
        let mut out: Vec<RolloutEstimate> = Vec::new();
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

pub fn new_memory_rollout_estimate_store() -> Rc<dyn RolloutEstimateStore> {
    Rc::new(MemoryRolloutEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRolloutEstimateStore {
    pub inner: Rc<dyn RolloutEstimateStore>,
    pub cache: RefCell<cache::TtlCache<RolloutEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RolloutEstimateStore for CachedRolloutEstimateStore {
    fn find(&self, code: &str) -> Option<RolloutEstimate> {
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

    fn save(&self, item: &RolloutEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RolloutEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_rollout_estimate_store(
    inner: Rc<dyn RolloutEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RolloutEstimateStore> {
    Rc::new(CachedRolloutEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RolloutEstimate>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollout estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEstimateQuery {
    pub source: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RolloutEstimateQuery {
    pub fn matches(&self, item: &RolloutEstimate) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[RolloutEstimate]) -> Vec<RolloutEstimate> {
        let matched = collections::filter(items, |candidate: &RolloutEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollout_estimates(
    store: Rc<dyn RolloutEstimateStore>,
    query: &RolloutEstimateQuery,
) -> Vec<RolloutEstimate> {
    query.apply(&store.list())
}
