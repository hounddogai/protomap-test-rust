use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention bundles by code.
pub trait RetentionBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionBundle>;
    fn save(&self, item: &RetentionBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionBundleStore {
    pub items: RefCell<BTreeMap<String, RetentionBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionBundleStore for MemoryRetentionBundleStore {
    fn find(&self, code: &str) -> Option<RetentionBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionBundle) {
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

    fn list(&self) -> Vec<RetentionBundle> {
        let mut out: Vec<RetentionBundle> = Vec::new();
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

pub fn new_memory_retention_bundle_store() -> Rc<dyn RetentionBundleStore> {
    Rc::new(MemoryRetentionBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionBundleStore {
    pub inner: Rc<dyn RetentionBundleStore>,
    pub cache: RefCell<cache::TtlCache<RetentionBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionBundleStore for CachedRetentionBundleStore {
    fn find(&self, code: &str) -> Option<RetentionBundle> {
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

    fn save(&self, item: &RetentionBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionBundle> {
        self.inner.list()
    }
}

pub fn new_cached_retention_bundle_store(
    inner: Rc<dyn RetentionBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionBundleStore> {
    Rc::new(CachedRetentionBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionBundle>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBundleQuery {
    pub category: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionBundleQuery {
    pub fn matches(&self, item: &RetentionBundle) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RetentionBundle]) -> Vec<RetentionBundle> {
        let matched = collections::filter(items, |candidate: &RetentionBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_bundles(
    store: Rc<dyn RetentionBundleStore>,
    query: &RetentionBundleQuery,
) -> Vec<RetentionBundle> {
    query.apply(&store.list())
}
