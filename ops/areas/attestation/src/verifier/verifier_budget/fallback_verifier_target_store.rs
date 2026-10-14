use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback verifier targets by code.
pub trait FallbackVerifierTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackVerifierTarget>;
    fn save(&self, item: &FallbackVerifierTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackVerifierTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackVerifierTargetStore {
    pub items: RefCell<BTreeMap<String, FallbackVerifierTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackVerifierTargetStore for MemoryFallbackVerifierTargetStore {
    fn find(&self, code: &str) -> Option<FallbackVerifierTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackVerifierTarget) {
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

    fn list(&self) -> Vec<FallbackVerifierTarget> {
        let mut out: Vec<FallbackVerifierTarget> = Vec::new();
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

pub fn new_memory_fallback_verifier_target_store() -> Rc<dyn FallbackVerifierTargetStore> {
    Rc::new(MemoryFallbackVerifierTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackVerifierTargetStore {
    pub inner: Rc<dyn FallbackVerifierTargetStore>,
    pub cache: RefCell<cache::TtlCache<FallbackVerifierTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackVerifierTargetStore for CachedFallbackVerifierTargetStore {
    fn find(&self, code: &str) -> Option<FallbackVerifierTarget> {
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

    fn save(&self, item: &FallbackVerifierTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackVerifierTarget> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_verifier_target_store(
    inner: Rc<dyn FallbackVerifierTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackVerifierTargetStore> {
    Rc::new(CachedFallbackVerifierTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackVerifierTarget>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback verifier targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierTargetQuery {
    pub cluster: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackVerifierTargetQuery {
    pub fn matches(&self, item: &FallbackVerifierTarget) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackVerifierTarget]) -> Vec<FallbackVerifierTarget> {
        let matched = collections::filter(items, |candidate: &FallbackVerifierTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_verifier_targets(
    store: Rc<dyn FallbackVerifierTargetStore>,
    query: &FallbackVerifierTargetQuery,
) -> Vec<FallbackVerifierTarget> {
    query.apply(&store.list())
}
