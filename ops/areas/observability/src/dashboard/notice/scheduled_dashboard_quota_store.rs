use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled dashboard quotas by code.
pub trait ScheduledDashboardQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledDashboardQuota>;
    fn save(&self, item: &ScheduledDashboardQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledDashboardQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledDashboardQuotaStore {
    pub items: RefCell<BTreeMap<String, ScheduledDashboardQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledDashboardQuotaStore for MemoryScheduledDashboardQuotaStore {
    fn find(&self, code: &str) -> Option<ScheduledDashboardQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledDashboardQuota) {
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

    fn list(&self) -> Vec<ScheduledDashboardQuota> {
        let mut out: Vec<ScheduledDashboardQuota> = Vec::new();
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

pub fn new_memory_scheduled_dashboard_quota_store() -> Rc<dyn ScheduledDashboardQuotaStore> {
    Rc::new(MemoryScheduledDashboardQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledDashboardQuotaStore {
    pub inner: Rc<dyn ScheduledDashboardQuotaStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledDashboardQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledDashboardQuotaStore for CachedScheduledDashboardQuotaStore {
    fn find(&self, code: &str) -> Option<ScheduledDashboardQuota> {
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

    fn save(&self, item: &ScheduledDashboardQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledDashboardQuota> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_dashboard_quota_store(
    inner: Rc<dyn ScheduledDashboardQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledDashboardQuotaStore> {
    Rc::new(CachedScheduledDashboardQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledDashboardQuota>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled dashboard quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardQuotaQuery {
    pub scope: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledDashboardQuotaQuery {
    pub fn matches(&self, item: &ScheduledDashboardQuota) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledDashboardQuota]) -> Vec<ScheduledDashboardQuota> {
        let matched = collections::filter(items, |candidate: &ScheduledDashboardQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_dashboard_quotas(
    store: Rc<dyn ScheduledDashboardQuotaStore>,
    query: &ScheduledDashboardQuotaQuery,
) -> Vec<ScheduledDashboardQuota> {
    query.apply(&store.list())
}
