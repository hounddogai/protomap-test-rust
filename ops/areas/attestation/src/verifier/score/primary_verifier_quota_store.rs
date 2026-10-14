use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary verifier quotas by code.
pub trait PrimaryVerifierQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryVerifierQuota>;
    fn save(&self, item: &PrimaryVerifierQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryVerifierQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryVerifierQuotaStore {
    pub items: RefCell<BTreeMap<String, PrimaryVerifierQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryVerifierQuotaStore for MemoryPrimaryVerifierQuotaStore {
    fn find(&self, code: &str) -> Option<PrimaryVerifierQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryVerifierQuota) {
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

    fn list(&self) -> Vec<PrimaryVerifierQuota> {
        let mut out: Vec<PrimaryVerifierQuota> = Vec::new();
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

pub fn new_memory_primary_verifier_quota_store() -> Rc<dyn PrimaryVerifierQuotaStore> {
    Rc::new(MemoryPrimaryVerifierQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryVerifierQuotaStore {
    pub inner: Rc<dyn PrimaryVerifierQuotaStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryVerifierQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryVerifierQuotaStore for CachedPrimaryVerifierQuotaStore {
    fn find(&self, code: &str) -> Option<PrimaryVerifierQuota> {
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

    fn save(&self, item: &PrimaryVerifierQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryVerifierQuota> {
        self.inner.list()
    }
}

pub fn new_cached_primary_verifier_quota_store(
    inner: Rc<dyn PrimaryVerifierQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryVerifierQuotaStore> {
    Rc::new(CachedPrimaryVerifierQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryVerifierQuota>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary verifier quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierQuotaQuery {
    pub channel: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryVerifierQuotaQuery {
    pub fn matches(&self, item: &PrimaryVerifierQuota) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryVerifierQuota]) -> Vec<PrimaryVerifierQuota> {
        let matched = collections::filter(items, |candidate: &PrimaryVerifierQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_verifier_quotas(
    store: Rc<dyn PrimaryVerifierQuotaStore>,
    query: &PrimaryVerifierQuotaQuery,
) -> Vec<PrimaryVerifierQuota> {
    query.apply(&store.list())
}
