use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps allowlist estimates by code.
pub trait AllowlistEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AllowlistEstimate>;
    fn save(&self, item: &AllowlistEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AllowlistEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAllowlistEstimateStore {
    pub items: RefCell<BTreeMap<String, AllowlistEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl AllowlistEstimateStore for MemoryAllowlistEstimateStore {
    fn find(&self, code: &str) -> Option<AllowlistEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AllowlistEstimate) {
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

    fn list(&self) -> Vec<AllowlistEstimate> {
        let mut out: Vec<AllowlistEstimate> = Vec::new();
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

pub fn new_memory_allowlist_estimate_store() -> Rc<dyn AllowlistEstimateStore> {
    Rc::new(MemoryAllowlistEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAllowlistEstimateStore {
    pub inner: Rc<dyn AllowlistEstimateStore>,
    pub cache: RefCell<cache::TtlCache<AllowlistEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AllowlistEstimateStore for CachedAllowlistEstimateStore {
    fn find(&self, code: &str) -> Option<AllowlistEstimate> {
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

    fn save(&self, item: &AllowlistEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AllowlistEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_allowlist_estimate_store(
    inner: Rc<dyn AllowlistEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AllowlistEstimateStore> {
    Rc::new(CachedAllowlistEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AllowlistEstimate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects allowlist estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEstimateQuery {
    pub title: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AllowlistEstimateQuery {
    pub fn matches(&self, item: &AllowlistEstimate) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[AllowlistEstimate]) -> Vec<AllowlistEstimate> {
        let matched = collections::filter(items, |candidate: &AllowlistEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_allowlist_estimates(
    store: Rc<dyn AllowlistEstimateStore>,
    query: &AllowlistEstimateQuery,
) -> Vec<AllowlistEstimate> {
    query.apply(&store.list())
}
