use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global plan quotas by code.
pub trait GlobalPlanQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalPlanQuota>;
    fn save(&self, item: &GlobalPlanQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalPlanQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalPlanQuotaStore {
    pub items: RefCell<BTreeMap<String, GlobalPlanQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalPlanQuotaStore for MemoryGlobalPlanQuotaStore {
    fn find(&self, code: &str) -> Option<GlobalPlanQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalPlanQuota) {
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

    fn list(&self) -> Vec<GlobalPlanQuota> {
        let mut out: Vec<GlobalPlanQuota> = Vec::new();
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

pub fn new_memory_global_plan_quota_store() -> Rc<dyn GlobalPlanQuotaStore> {
    Rc::new(MemoryGlobalPlanQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalPlanQuotaStore {
    pub inner: Rc<dyn GlobalPlanQuotaStore>,
    pub cache: RefCell<cache::TtlCache<GlobalPlanQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalPlanQuotaStore for CachedGlobalPlanQuotaStore {
    fn find(&self, code: &str) -> Option<GlobalPlanQuota> {
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

    fn save(&self, item: &GlobalPlanQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalPlanQuota> {
        self.inner.list()
    }
}

pub fn new_cached_global_plan_quota_store(
    inner: Rc<dyn GlobalPlanQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalPlanQuotaStore> {
    Rc::new(CachedGlobalPlanQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalPlanQuota>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global plan quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanQuotaQuery {
    pub source: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalPlanQuotaQuery {
    pub fn matches(&self, item: &GlobalPlanQuota) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalPlanQuota]) -> Vec<GlobalPlanQuota> {
        let matched = collections::filter(items, |candidate: &GlobalPlanQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_plan_quotas(
    store: Rc<dyn GlobalPlanQuotaStore>,
    query: &GlobalPlanQuotaQuery,
) -> Vec<GlobalPlanQuota> {
    query.apply(&store.list())
}
