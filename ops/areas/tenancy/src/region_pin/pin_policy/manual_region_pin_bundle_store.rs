use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual region pin bundles by code.
pub trait ManualRegionPinBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualRegionPinBundle>;
    fn save(&self, item: &ManualRegionPinBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualRegionPinBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualRegionPinBundleStore {
    pub items: RefCell<BTreeMap<String, ManualRegionPinBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualRegionPinBundleStore for MemoryManualRegionPinBundleStore {
    fn find(&self, code: &str) -> Option<ManualRegionPinBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualRegionPinBundle) {
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

    fn list(&self) -> Vec<ManualRegionPinBundle> {
        let mut out: Vec<ManualRegionPinBundle> = Vec::new();
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

pub fn new_memory_manual_region_pin_bundle_store() -> Rc<dyn ManualRegionPinBundleStore> {
    Rc::new(MemoryManualRegionPinBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualRegionPinBundleStore {
    pub inner: Rc<dyn ManualRegionPinBundleStore>,
    pub cache: RefCell<cache::TtlCache<ManualRegionPinBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualRegionPinBundleStore for CachedManualRegionPinBundleStore {
    fn find(&self, code: &str) -> Option<ManualRegionPinBundle> {
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

    fn save(&self, item: &ManualRegionPinBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualRegionPinBundle> {
        self.inner.list()
    }
}

pub fn new_cached_manual_region_pin_bundle_store(
    inner: Rc<dyn ManualRegionPinBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualRegionPinBundleStore> {
    Rc::new(CachedManualRegionPinBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualRegionPinBundle>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual region pin bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinBundleQuery {
    pub label: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualRegionPinBundleQuery {
    pub fn matches(&self, item: &ManualRegionPinBundle) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualRegionPinBundle]) -> Vec<ManualRegionPinBundle> {
        let matched = collections::filter(items, |candidate: &ManualRegionPinBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_region_pin_bundles(
    store: Rc<dyn ManualRegionPinBundleStore>,
    query: &ManualRegionPinBundleQuery,
) -> Vec<ManualRegionPinBundle> {
    query.apply(&store.list())
}
