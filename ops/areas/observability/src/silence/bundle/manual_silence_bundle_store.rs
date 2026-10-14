use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual silence bundles by code.
pub trait ManualSilenceBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualSilenceBundle>;
    fn save(&self, item: &ManualSilenceBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualSilenceBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualSilenceBundleStore {
    pub items: RefCell<BTreeMap<String, ManualSilenceBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualSilenceBundleStore for MemoryManualSilenceBundleStore {
    fn find(&self, code: &str) -> Option<ManualSilenceBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualSilenceBundle) {
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

    fn list(&self) -> Vec<ManualSilenceBundle> {
        let mut out: Vec<ManualSilenceBundle> = Vec::new();
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

pub fn new_memory_manual_silence_bundle_store() -> Rc<dyn ManualSilenceBundleStore> {
    Rc::new(MemoryManualSilenceBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualSilenceBundleStore {
    pub inner: Rc<dyn ManualSilenceBundleStore>,
    pub cache: RefCell<cache::TtlCache<ManualSilenceBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualSilenceBundleStore for CachedManualSilenceBundleStore {
    fn find(&self, code: &str) -> Option<ManualSilenceBundle> {
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

    fn save(&self, item: &ManualSilenceBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualSilenceBundle> {
        self.inner.list()
    }
}

pub fn new_cached_manual_silence_bundle_store(
    inner: Rc<dyn ManualSilenceBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualSilenceBundleStore> {
    Rc::new(CachedManualSilenceBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualSilenceBundle>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual silence bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceBundleQuery {
    pub namespace: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualSilenceBundleQuery {
    pub fn matches(&self, item: &ManualSilenceBundle) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[ManualSilenceBundle]) -> Vec<ManualSilenceBundle> {
        let matched = collections::filter(items, |candidate: &ManualSilenceBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_silence_bundles(
    store: Rc<dyn ManualSilenceBundleStore>,
    query: &ManualSilenceBundleQuery,
) -> Vec<ManualSilenceBundle> {
    query.apply(&store.list())
}
