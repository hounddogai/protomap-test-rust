use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dashboard quotas by code.
pub trait DashboardQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DashboardQuota>;
    fn save(&self, item: &DashboardQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DashboardQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDashboardQuotaStore {
    pub items: RefCell<BTreeMap<String, DashboardQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl DashboardQuotaStore for MemoryDashboardQuotaStore {
    fn find(&self, code: &str) -> Option<DashboardQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DashboardQuota) {
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

    fn list(&self) -> Vec<DashboardQuota> {
        let mut out: Vec<DashboardQuota> = Vec::new();
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

pub fn new_memory_dashboard_quota_store() -> Rc<dyn DashboardQuotaStore> {
    Rc::new(MemoryDashboardQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDashboardQuotaStore {
    pub inner: Rc<dyn DashboardQuotaStore>,
    pub cache: RefCell<cache::TtlCache<DashboardQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DashboardQuotaStore for CachedDashboardQuotaStore {
    fn find(&self, code: &str) -> Option<DashboardQuota> {
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

    fn save(&self, item: &DashboardQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DashboardQuota> {
        self.inner.list()
    }
}

pub fn new_cached_dashboard_quota_store(
    inner: Rc<dyn DashboardQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DashboardQuotaStore> {
    Rc::new(CachedDashboardQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DashboardQuota>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dashboard quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardQuotaQuery {
    pub name: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DashboardQuotaQuery {
    pub fn matches(&self, item: &DashboardQuota) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[DashboardQuota]) -> Vec<DashboardQuota> {
        let matched = collections::filter(items, |candidate: &DashboardQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dashboard_quotas(store: Rc<dyn DashboardQuotaStore>, query: &DashboardQuotaQuery) -> Vec<DashboardQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DashboardQuotaStore> = new_memory_dashboard_quota_store();
        let item = new_dashboard_quota("code-816", "name-375");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
