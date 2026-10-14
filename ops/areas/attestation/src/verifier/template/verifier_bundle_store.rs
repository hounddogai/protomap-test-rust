use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps verifier bundles by code.
pub trait VerifierBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<VerifierBundle>;
    fn save(&self, item: &VerifierBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<VerifierBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryVerifierBundleStore {
    pub items: RefCell<BTreeMap<String, VerifierBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl VerifierBundleStore for MemoryVerifierBundleStore {
    fn find(&self, code: &str) -> Option<VerifierBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &VerifierBundle) {
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

    fn list(&self) -> Vec<VerifierBundle> {
        let mut out: Vec<VerifierBundle> = Vec::new();
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

pub fn new_memory_verifier_bundle_store() -> Rc<dyn VerifierBundleStore> {
    Rc::new(MemoryVerifierBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedVerifierBundleStore {
    pub inner: Rc<dyn VerifierBundleStore>,
    pub cache: RefCell<cache::TtlCache<VerifierBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl VerifierBundleStore for CachedVerifierBundleStore {
    fn find(&self, code: &str) -> Option<VerifierBundle> {
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

    fn save(&self, item: &VerifierBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<VerifierBundle> {
        self.inner.list()
    }
}

pub fn new_cached_verifier_bundle_store(
    inner: Rc<dyn VerifierBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn VerifierBundleStore> {
    Rc::new(CachedVerifierBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<VerifierBundle>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects verifier bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBundleQuery {
    pub source: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl VerifierBundleQuery {
    pub fn matches(&self, item: &VerifierBundle) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[VerifierBundle]) -> Vec<VerifierBundle> {
        let matched = collections::filter(items, |candidate: &VerifierBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_verifier_bundles(store: Rc<dyn VerifierBundleStore>, query: &VerifierBundleQuery) -> Vec<VerifierBundle> {
    query.apply(&store.list())
}
