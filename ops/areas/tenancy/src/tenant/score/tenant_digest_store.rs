use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tenant digests by code.
pub trait TenantDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TenantDigest>;
    fn save(&self, item: &TenantDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TenantDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTenantDigestStore {
    pub items: RefCell<BTreeMap<String, TenantDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl TenantDigestStore for MemoryTenantDigestStore {
    fn find(&self, code: &str) -> Option<TenantDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TenantDigest) {
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

    fn list(&self) -> Vec<TenantDigest> {
        let mut out: Vec<TenantDigest> = Vec::new();
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

pub fn new_memory_tenant_digest_store() -> Rc<dyn TenantDigestStore> {
    Rc::new(MemoryTenantDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTenantDigestStore {
    pub inner: Rc<dyn TenantDigestStore>,
    pub cache: RefCell<cache::TtlCache<TenantDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TenantDigestStore for CachedTenantDigestStore {
    fn find(&self, code: &str) -> Option<TenantDigest> {
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

    fn save(&self, item: &TenantDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TenantDigest> {
        self.inner.list()
    }
}

pub fn new_cached_tenant_digest_store(
    inner: Rc<dyn TenantDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TenantDigestStore> {
    Rc::new(CachedTenantDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TenantDigest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tenant digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantDigestQuery {
    pub locale: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TenantDigestQuery {
    pub fn matches(&self, item: &TenantDigest) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TenantDigest]) -> Vec<TenantDigest> {
        let matched = collections::filter(items, |candidate: &TenantDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tenant_digests(store: Rc<dyn TenantDigestStore>, query: &TenantDigestQuery) -> Vec<TenantDigest> {
    query.apply(&store.list())
}
