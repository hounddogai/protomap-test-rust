use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted federation bundles by code.
pub trait WeightedFederationBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedFederationBundle>;
    fn save(&self, item: &WeightedFederationBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedFederationBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedFederationBundleStore {
    pub items: RefCell<BTreeMap<String, WeightedFederationBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedFederationBundleStore for MemoryWeightedFederationBundleStore {
    fn find(&self, code: &str) -> Option<WeightedFederationBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedFederationBundle) {
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

    fn list(&self) -> Vec<WeightedFederationBundle> {
        let mut out: Vec<WeightedFederationBundle> = Vec::new();
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

pub fn new_memory_weighted_federation_bundle_store() -> Rc<dyn WeightedFederationBundleStore> {
    Rc::new(MemoryWeightedFederationBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedFederationBundleStore {
    pub inner: Rc<dyn WeightedFederationBundleStore>,
    pub cache: RefCell<cache::TtlCache<WeightedFederationBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedFederationBundleStore for CachedWeightedFederationBundleStore {
    fn find(&self, code: &str) -> Option<WeightedFederationBundle> {
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

    fn save(&self, item: &WeightedFederationBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedFederationBundle> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_federation_bundle_store(
    inner: Rc<dyn WeightedFederationBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedFederationBundleStore> {
    Rc::new(CachedWeightedFederationBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedFederationBundle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted federation bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationBundleQuery {
    pub category: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedFederationBundleQuery {
    pub fn matches(&self, item: &WeightedFederationBundle) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedFederationBundle]) -> Vec<WeightedFederationBundle> {
        let matched = collections::filter(items, |candidate: &WeightedFederationBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_federation_bundles(
    store: Rc<dyn WeightedFederationBundleStore>,
    query: &WeightedFederationBundleQuery,
) -> Vec<WeightedFederationBundle> {
    query.apply(&store.list())
}
