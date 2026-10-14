use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped redaction bundles by code.
pub trait CappedRedactionBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedRedactionBundle>;
    fn save(&self, item: &CappedRedactionBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedRedactionBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedRedactionBundleStore {
    pub items: RefCell<BTreeMap<String, CappedRedactionBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedRedactionBundleStore for MemoryCappedRedactionBundleStore {
    fn find(&self, code: &str) -> Option<CappedRedactionBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedRedactionBundle) {
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

    fn list(&self) -> Vec<CappedRedactionBundle> {
        let mut out: Vec<CappedRedactionBundle> = Vec::new();
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

pub fn new_memory_capped_redaction_bundle_store() -> Rc<dyn CappedRedactionBundleStore> {
    Rc::new(MemoryCappedRedactionBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedRedactionBundleStore {
    pub inner: Rc<dyn CappedRedactionBundleStore>,
    pub cache: RefCell<cache::TtlCache<CappedRedactionBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedRedactionBundleStore for CachedCappedRedactionBundleStore {
    fn find(&self, code: &str) -> Option<CappedRedactionBundle> {
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

    fn save(&self, item: &CappedRedactionBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedRedactionBundle> {
        self.inner.list()
    }
}

pub fn new_cached_capped_redaction_bundle_store(
    inner: Rc<dyn CappedRedactionBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedRedactionBundleStore> {
    Rc::new(CachedCappedRedactionBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedRedactionBundle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped redaction bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionBundleQuery {
    pub label: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedRedactionBundleQuery {
    pub fn matches(&self, item: &CappedRedactionBundle) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedRedactionBundle]) -> Vec<CappedRedactionBundle> {
        let matched = collections::filter(items, |candidate: &CappedRedactionBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_redaction_bundles(
    store: Rc<dyn CappedRedactionBundleStore>,
    query: &CappedRedactionBundleQuery,
) -> Vec<CappedRedactionBundle> {
    query.apply(&store.list())
}
