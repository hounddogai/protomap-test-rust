use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted credential variances by code.
pub trait WeightedCredentialVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedCredentialVariance>;
    fn save(&self, item: &WeightedCredentialVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedCredentialVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedCredentialVarianceStore {
    pub items: RefCell<BTreeMap<String, WeightedCredentialVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedCredentialVarianceStore for MemoryWeightedCredentialVarianceStore {
    fn find(&self, code: &str) -> Option<WeightedCredentialVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedCredentialVariance) {
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

    fn list(&self) -> Vec<WeightedCredentialVariance> {
        let mut out: Vec<WeightedCredentialVariance> = Vec::new();
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

pub fn new_memory_weighted_credential_variance_store() -> Rc<dyn WeightedCredentialVarianceStore> {
    Rc::new(MemoryWeightedCredentialVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedCredentialVarianceStore {
    pub inner: Rc<dyn WeightedCredentialVarianceStore>,
    pub cache: RefCell<cache::TtlCache<WeightedCredentialVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedCredentialVarianceStore for CachedWeightedCredentialVarianceStore {
    fn find(&self, code: &str) -> Option<WeightedCredentialVariance> {
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

    fn save(&self, item: &WeightedCredentialVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedCredentialVariance> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_credential_variance_store(
    inner: Rc<dyn WeightedCredentialVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedCredentialVarianceStore> {
    Rc::new(CachedWeightedCredentialVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedCredentialVariance>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted credential variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialVarianceQuery {
    pub channel: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedCredentialVarianceQuery {
    pub fn matches(&self, item: &WeightedCredentialVariance) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[WeightedCredentialVariance]) -> Vec<WeightedCredentialVariance> {
        let matched = collections::filter(items, |candidate: &WeightedCredentialVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_credential_variances(
    store: Rc<dyn WeightedCredentialVarianceStore>,
    query: &WeightedCredentialVarianceQuery,
) -> Vec<WeightedCredentialVariance> {
    query.apply(&store.list())
}
