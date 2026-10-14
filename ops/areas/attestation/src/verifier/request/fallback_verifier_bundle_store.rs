use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback verifier bundles by code.
pub trait FallbackVerifierBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackVerifierBundle>;
    fn save(&self, item: &FallbackVerifierBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackVerifierBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackVerifierBundleStore {
    pub items: RefCell<BTreeMap<String, FallbackVerifierBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackVerifierBundleStore for MemoryFallbackVerifierBundleStore {
    fn find(&self, code: &str) -> Option<FallbackVerifierBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackVerifierBundle) {
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

    fn list(&self) -> Vec<FallbackVerifierBundle> {
        let mut out: Vec<FallbackVerifierBundle> = Vec::new();
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

pub fn new_memory_fallback_verifier_bundle_store() -> Rc<dyn FallbackVerifierBundleStore> {
    Rc::new(MemoryFallbackVerifierBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackVerifierBundleStore {
    pub inner: Rc<dyn FallbackVerifierBundleStore>,
    pub cache: RefCell<cache::TtlCache<FallbackVerifierBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackVerifierBundleStore for CachedFallbackVerifierBundleStore {
    fn find(&self, code: &str) -> Option<FallbackVerifierBundle> {
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

    fn save(&self, item: &FallbackVerifierBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackVerifierBundle> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_verifier_bundle_store(
    inner: Rc<dyn FallbackVerifierBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackVerifierBundleStore> {
    Rc::new(CachedFallbackVerifierBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackVerifierBundle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback verifier bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierBundleQuery {
    pub namespace: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackVerifierBundleQuery {
    pub fn matches(&self, item: &FallbackVerifierBundle) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackVerifierBundle]) -> Vec<FallbackVerifierBundle> {
        let matched = collections::filter(items, |candidate: &FallbackVerifierBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_verifier_bundles(
    store: Rc<dyn FallbackVerifierBundleStore>,
    query: &FallbackVerifierBundleQuery,
) -> Vec<FallbackVerifierBundle> {
    query.apply(&store.list())
}
