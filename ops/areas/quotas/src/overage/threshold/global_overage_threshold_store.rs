use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global overage thresholds by code.
pub trait GlobalOverageThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalOverageThreshold>;
    fn save(&self, item: &GlobalOverageThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalOverageThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalOverageThresholdStore {
    pub items: RefCell<BTreeMap<String, GlobalOverageThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalOverageThresholdStore for MemoryGlobalOverageThresholdStore {
    fn find(&self, code: &str) -> Option<GlobalOverageThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalOverageThreshold) {
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

    fn list(&self) -> Vec<GlobalOverageThreshold> {
        let mut out: Vec<GlobalOverageThreshold> = Vec::new();
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

pub fn new_memory_global_overage_threshold_store() -> Rc<dyn GlobalOverageThresholdStore> {
    Rc::new(MemoryGlobalOverageThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalOverageThresholdStore {
    pub inner: Rc<dyn GlobalOverageThresholdStore>,
    pub cache: RefCell<cache::TtlCache<GlobalOverageThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalOverageThresholdStore for CachedGlobalOverageThresholdStore {
    fn find(&self, code: &str) -> Option<GlobalOverageThreshold> {
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

    fn save(&self, item: &GlobalOverageThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalOverageThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_global_overage_threshold_store(
    inner: Rc<dyn GlobalOverageThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalOverageThresholdStore> {
    Rc::new(CachedGlobalOverageThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalOverageThreshold>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global overage thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageThresholdQuery {
    pub cluster: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalOverageThresholdQuery {
    pub fn matches(&self, item: &GlobalOverageThreshold) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalOverageThreshold]) -> Vec<GlobalOverageThreshold> {
        let matched = collections::filter(items, |candidate: &GlobalOverageThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_overage_thresholds(
    store: Rc<dyn GlobalOverageThresholdStore>,
    query: &GlobalOverageThresholdQuery,
) -> Vec<GlobalOverageThreshold> {
    query.apply(&store.list())
}
