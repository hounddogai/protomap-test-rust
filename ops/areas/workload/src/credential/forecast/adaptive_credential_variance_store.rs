use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive credential variances by code.
pub trait AdaptiveCredentialVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveCredentialVariance>;
    fn save(&self, item: &AdaptiveCredentialVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveCredentialVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveCredentialVarianceStore {
    pub items: RefCell<BTreeMap<String, AdaptiveCredentialVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveCredentialVarianceStore for MemoryAdaptiveCredentialVarianceStore {
    fn find(&self, code: &str) -> Option<AdaptiveCredentialVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveCredentialVariance) {
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

    fn list(&self) -> Vec<AdaptiveCredentialVariance> {
        let mut out: Vec<AdaptiveCredentialVariance> = Vec::new();
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

pub fn new_memory_adaptive_credential_variance_store() -> Rc<dyn AdaptiveCredentialVarianceStore> {
    Rc::new(MemoryAdaptiveCredentialVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveCredentialVarianceStore {
    pub inner: Rc<dyn AdaptiveCredentialVarianceStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveCredentialVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveCredentialVarianceStore for CachedAdaptiveCredentialVarianceStore {
    fn find(&self, code: &str) -> Option<AdaptiveCredentialVariance> {
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

    fn save(&self, item: &AdaptiveCredentialVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveCredentialVariance> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_credential_variance_store(
    inner: Rc<dyn AdaptiveCredentialVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveCredentialVarianceStore> {
    Rc::new(CachedAdaptiveCredentialVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveCredentialVariance>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive credential variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialVarianceQuery {
    pub unit: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveCredentialVarianceQuery {
    pub fn matches(&self, item: &AdaptiveCredentialVariance) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveCredentialVariance]) -> Vec<AdaptiveCredentialVariance> {
        let matched = collections::filter(items, |candidate: &AdaptiveCredentialVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_credential_variances(
    store: Rc<dyn AdaptiveCredentialVarianceStore>,
    query: &AdaptiveCredentialVarianceQuery,
) -> Vec<AdaptiveCredentialVariance> {
    query.apply(&store.list())
}
