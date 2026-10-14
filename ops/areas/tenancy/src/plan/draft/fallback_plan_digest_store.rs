use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback plan digests by code.
pub trait FallbackPlanDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackPlanDigest>;
    fn save(&self, item: &FallbackPlanDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackPlanDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackPlanDigestStore {
    pub items: RefCell<BTreeMap<String, FallbackPlanDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackPlanDigestStore for MemoryFallbackPlanDigestStore {
    fn find(&self, code: &str) -> Option<FallbackPlanDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackPlanDigest) {
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

    fn list(&self) -> Vec<FallbackPlanDigest> {
        let mut out: Vec<FallbackPlanDigest> = Vec::new();
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

pub fn new_memory_fallback_plan_digest_store() -> Rc<dyn FallbackPlanDigestStore> {
    Rc::new(MemoryFallbackPlanDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackPlanDigestStore {
    pub inner: Rc<dyn FallbackPlanDigestStore>,
    pub cache: RefCell<cache::TtlCache<FallbackPlanDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackPlanDigestStore for CachedFallbackPlanDigestStore {
    fn find(&self, code: &str) -> Option<FallbackPlanDigest> {
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

    fn save(&self, item: &FallbackPlanDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackPlanDigest> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_plan_digest_store(
    inner: Rc<dyn FallbackPlanDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackPlanDigestStore> {
    Rc::new(CachedFallbackPlanDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackPlanDigest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback plan digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanDigestQuery {
    pub category: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackPlanDigestQuery {
    pub fn matches(&self, item: &FallbackPlanDigest) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackPlanDigest]) -> Vec<FallbackPlanDigest> {
        let matched = collections::filter(items, |candidate: &FallbackPlanDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_plan_digests(
    store: Rc<dyn FallbackPlanDigestStore>,
    query: &FallbackPlanDigestQuery,
) -> Vec<FallbackPlanDigest> {
    query.apply(&store.list())
}
