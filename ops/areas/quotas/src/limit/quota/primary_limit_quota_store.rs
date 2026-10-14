use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary limit quotas by code.
pub trait PrimaryLimitQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryLimitQuota>;
    fn save(&self, item: &PrimaryLimitQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryLimitQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryLimitQuotaStore {
    pub items: RefCell<BTreeMap<String, PrimaryLimitQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryLimitQuotaStore for MemoryPrimaryLimitQuotaStore {
    fn find(&self, code: &str) -> Option<PrimaryLimitQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryLimitQuota) {
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

    fn list(&self) -> Vec<PrimaryLimitQuota> {
        let mut out: Vec<PrimaryLimitQuota> = Vec::new();
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

pub fn new_memory_primary_limit_quota_store() -> Rc<dyn PrimaryLimitQuotaStore> {
    Rc::new(MemoryPrimaryLimitQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryLimitQuotaStore {
    pub inner: Rc<dyn PrimaryLimitQuotaStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryLimitQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryLimitQuotaStore for CachedPrimaryLimitQuotaStore {
    fn find(&self, code: &str) -> Option<PrimaryLimitQuota> {
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

    fn save(&self, item: &PrimaryLimitQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryLimitQuota> {
        self.inner.list()
    }
}

pub fn new_cached_primary_limit_quota_store(
    inner: Rc<dyn PrimaryLimitQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryLimitQuotaStore> {
    Rc::new(CachedPrimaryLimitQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryLimitQuota>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary limit quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitQuotaQuery {
    pub category: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryLimitQuotaQuery {
    pub fn matches(&self, item: &PrimaryLimitQuota) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[PrimaryLimitQuota]) -> Vec<PrimaryLimitQuota> {
        let matched = collections::filter(items, |candidate: &PrimaryLimitQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_limit_quotas(
    store: Rc<dyn PrimaryLimitQuotaStore>,
    query: &PrimaryLimitQuotaQuery,
) -> Vec<PrimaryLimitQuota> {
    query.apply(&store.list())
}
