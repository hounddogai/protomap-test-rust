use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal page quotas by code.
pub trait SeasonalPageQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalPageQuota>;
    fn save(&self, item: &SeasonalPageQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalPageQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalPageQuotaStore {
    pub items: RefCell<BTreeMap<String, SeasonalPageQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalPageQuotaStore for MemorySeasonalPageQuotaStore {
    fn find(&self, code: &str) -> Option<SeasonalPageQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalPageQuota) {
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

    fn list(&self) -> Vec<SeasonalPageQuota> {
        let mut out: Vec<SeasonalPageQuota> = Vec::new();
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

pub fn new_memory_seasonal_page_quota_store() -> Rc<dyn SeasonalPageQuotaStore> {
    Rc::new(MemorySeasonalPageQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalPageQuotaStore {
    pub inner: Rc<dyn SeasonalPageQuotaStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalPageQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalPageQuotaStore for CachedSeasonalPageQuotaStore {
    fn find(&self, code: &str) -> Option<SeasonalPageQuota> {
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

    fn save(&self, item: &SeasonalPageQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalPageQuota> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_page_quota_store(
    inner: Rc<dyn SeasonalPageQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalPageQuotaStore> {
    Rc::new(CachedSeasonalPageQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalPageQuota>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal page quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageQuotaQuery {
    pub cluster: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalPageQuotaQuery {
    pub fn matches(&self, item: &SeasonalPageQuota) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[SeasonalPageQuota]) -> Vec<SeasonalPageQuota> {
        let matched = collections::filter(items, |candidate: &SeasonalPageQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_page_quotas(
    store: Rc<dyn SeasonalPageQuotaStore>,
    query: &SeasonalPageQuotaQuery,
) -> Vec<SeasonalPageQuota> {
    query.apply(&store.list())
}
