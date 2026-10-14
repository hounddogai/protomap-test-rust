use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps trust domain thresholds by code.
pub trait TrustDomainThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TrustDomainThreshold>;
    fn save(&self, item: &TrustDomainThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TrustDomainThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTrustDomainThresholdStore {
    pub items: RefCell<BTreeMap<String, TrustDomainThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl TrustDomainThresholdStore for MemoryTrustDomainThresholdStore {
    fn find(&self, code: &str) -> Option<TrustDomainThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TrustDomainThreshold) {
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

    fn list(&self) -> Vec<TrustDomainThreshold> {
        let mut out: Vec<TrustDomainThreshold> = Vec::new();
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

pub fn new_memory_trust_domain_threshold_store() -> Rc<dyn TrustDomainThresholdStore> {
    Rc::new(MemoryTrustDomainThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTrustDomainThresholdStore {
    pub inner: Rc<dyn TrustDomainThresholdStore>,
    pub cache: RefCell<cache::TtlCache<TrustDomainThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TrustDomainThresholdStore for CachedTrustDomainThresholdStore {
    fn find(&self, code: &str) -> Option<TrustDomainThreshold> {
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

    fn save(&self, item: &TrustDomainThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TrustDomainThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_trust_domain_threshold_store(
    inner: Rc<dyn TrustDomainThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TrustDomainThresholdStore> {
    Rc::new(CachedTrustDomainThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TrustDomainThreshold>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects trust domain thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainThresholdQuery {
    pub region: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TrustDomainThresholdQuery {
    pub fn matches(&self, item: &TrustDomainThreshold) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TrustDomainThreshold]) -> Vec<TrustDomainThreshold> {
        let matched = collections::filter(items, |candidate: &TrustDomainThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_trust_domain_thresholds(
    store: Rc<dyn TrustDomainThresholdStore>,
    query: &TrustDomainThresholdQuery,
) -> Vec<TrustDomainThreshold> {
    query.apply(&store.list())
}
