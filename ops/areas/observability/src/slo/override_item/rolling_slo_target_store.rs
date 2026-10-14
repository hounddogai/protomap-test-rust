use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling slo targets by code.
pub trait RollingSloTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingSloTarget>;
    fn save(&self, item: &RollingSloTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingSloTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingSloTargetStore {
    pub items: RefCell<BTreeMap<String, RollingSloTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingSloTargetStore for MemoryRollingSloTargetStore {
    fn find(&self, code: &str) -> Option<RollingSloTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingSloTarget) {
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

    fn list(&self) -> Vec<RollingSloTarget> {
        let mut out: Vec<RollingSloTarget> = Vec::new();
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

pub fn new_memory_rolling_slo_target_store() -> Rc<dyn RollingSloTargetStore> {
    Rc::new(MemoryRollingSloTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingSloTargetStore {
    pub inner: Rc<dyn RollingSloTargetStore>,
    pub cache: RefCell<cache::TtlCache<RollingSloTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingSloTargetStore for CachedRollingSloTargetStore {
    fn find(&self, code: &str) -> Option<RollingSloTarget> {
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

    fn save(&self, item: &RollingSloTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingSloTarget> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_slo_target_store(
    inner: Rc<dyn RollingSloTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingSloTargetStore> {
    Rc::new(CachedRollingSloTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingSloTarget>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling slo targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloTargetQuery {
    pub cluster: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingSloTargetQuery {
    pub fn matches(&self, item: &RollingSloTarget) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingSloTarget]) -> Vec<RollingSloTarget> {
        let matched = collections::filter(items, |candidate: &RollingSloTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_slo_targets(
    store: Rc<dyn RollingSloTargetStore>,
    query: &RollingSloTargetQuery,
) -> Vec<RollingSloTarget> {
    query.apply(&store.list())
}
