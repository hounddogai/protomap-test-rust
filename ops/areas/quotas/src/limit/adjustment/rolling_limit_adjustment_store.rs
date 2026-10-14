use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling limit adjustments by code.
pub trait RollingLimitAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingLimitAdjustment>;
    fn save(&self, item: &RollingLimitAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingLimitAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingLimitAdjustmentStore {
    pub items: RefCell<BTreeMap<String, RollingLimitAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingLimitAdjustmentStore for MemoryRollingLimitAdjustmentStore {
    fn find(&self, code: &str) -> Option<RollingLimitAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingLimitAdjustment) {
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

    fn list(&self) -> Vec<RollingLimitAdjustment> {
        let mut out: Vec<RollingLimitAdjustment> = Vec::new();
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

pub fn new_memory_rolling_limit_adjustment_store() -> Rc<dyn RollingLimitAdjustmentStore> {
    Rc::new(MemoryRollingLimitAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingLimitAdjustmentStore {
    pub inner: Rc<dyn RollingLimitAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<RollingLimitAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingLimitAdjustmentStore for CachedRollingLimitAdjustmentStore {
    fn find(&self, code: &str) -> Option<RollingLimitAdjustment> {
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

    fn save(&self, item: &RollingLimitAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingLimitAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_limit_adjustment_store(
    inner: Rc<dyn RollingLimitAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingLimitAdjustmentStore> {
    Rc::new(CachedRollingLimitAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingLimitAdjustment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling limit adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitAdjustmentQuery {
    pub country: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingLimitAdjustmentQuery {
    pub fn matches(&self, item: &RollingLimitAdjustment) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingLimitAdjustment]) -> Vec<RollingLimitAdjustment> {
        let matched = collections::filter(items, |candidate: &RollingLimitAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_limit_adjustments(
    store: Rc<dyn RollingLimitAdjustmentStore>,
    query: &RollingLimitAdjustmentQuery,
) -> Vec<RollingLimitAdjustment> {
    query.apply(&store.list())
}
