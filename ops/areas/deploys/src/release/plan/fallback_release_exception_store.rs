use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback release exceptions by code.
pub trait FallbackReleaseExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackReleaseException>;
    fn save(&self, item: &FallbackReleaseException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackReleaseException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackReleaseExceptionStore {
    pub items: RefCell<BTreeMap<String, FallbackReleaseException>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackReleaseExceptionStore for MemoryFallbackReleaseExceptionStore {
    fn find(&self, code: &str) -> Option<FallbackReleaseException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackReleaseException) {
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

    fn list(&self) -> Vec<FallbackReleaseException> {
        let mut out: Vec<FallbackReleaseException> = Vec::new();
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

pub fn new_memory_fallback_release_exception_store() -> Rc<dyn FallbackReleaseExceptionStore> {
    Rc::new(MemoryFallbackReleaseExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackReleaseExceptionStore {
    pub inner: Rc<dyn FallbackReleaseExceptionStore>,
    pub cache: RefCell<cache::TtlCache<FallbackReleaseException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackReleaseExceptionStore for CachedFallbackReleaseExceptionStore {
    fn find(&self, code: &str) -> Option<FallbackReleaseException> {
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

    fn save(&self, item: &FallbackReleaseException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackReleaseException> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_release_exception_store(
    inner: Rc<dyn FallbackReleaseExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackReleaseExceptionStore> {
    Rc::new(CachedFallbackReleaseExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackReleaseException>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback release exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseExceptionQuery {
    pub name: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackReleaseExceptionQuery {
    pub fn matches(&self, item: &FallbackReleaseException) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackReleaseException]) -> Vec<FallbackReleaseException> {
        let matched = collections::filter(items, |candidate: &FallbackReleaseException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_release_exceptions(
    store: Rc<dyn FallbackReleaseExceptionStore>,
    query: &FallbackReleaseExceptionQuery,
) -> Vec<FallbackReleaseException> {
    query.apply(&store.list())
}
