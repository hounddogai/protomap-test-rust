use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped binding quotas by code.
pub trait CappedBindingQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedBindingQuota>;
    fn save(&self, item: &CappedBindingQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedBindingQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedBindingQuotaStore {
    pub items: RefCell<BTreeMap<String, CappedBindingQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedBindingQuotaStore for MemoryCappedBindingQuotaStore {
    fn find(&self, code: &str) -> Option<CappedBindingQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedBindingQuota) {
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

    fn list(&self) -> Vec<CappedBindingQuota> {
        let mut out: Vec<CappedBindingQuota> = Vec::new();
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

pub fn new_memory_capped_binding_quota_store() -> Rc<dyn CappedBindingQuotaStore> {
    Rc::new(MemoryCappedBindingQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedBindingQuotaStore {
    pub inner: Rc<dyn CappedBindingQuotaStore>,
    pub cache: RefCell<cache::TtlCache<CappedBindingQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedBindingQuotaStore for CachedCappedBindingQuotaStore {
    fn find(&self, code: &str) -> Option<CappedBindingQuota> {
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

    fn save(&self, item: &CappedBindingQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedBindingQuota> {
        self.inner.list()
    }
}

pub fn new_cached_capped_binding_quota_store(
    inner: Rc<dyn CappedBindingQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedBindingQuotaStore> {
    Rc::new(CachedCappedBindingQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedBindingQuota>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped binding quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBindingQuotaQuery {
    pub label: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedBindingQuotaQuery {
    pub fn matches(&self, item: &CappedBindingQuota) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedBindingQuota]) -> Vec<CappedBindingQuota> {
        let matched = collections::filter(items, |candidate: &CappedBindingQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_binding_quotas(
    store: Rc<dyn CappedBindingQuotaStore>,
    query: &CappedBindingQuotaQuery,
) -> Vec<CappedBindingQuota> {
    query.apply(&store.list())
}
