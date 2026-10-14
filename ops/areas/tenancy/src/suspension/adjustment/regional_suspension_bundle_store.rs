use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional suspension bundles by code.
pub trait RegionalSuspensionBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalSuspensionBundle>;
    fn save(&self, item: &RegionalSuspensionBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalSuspensionBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalSuspensionBundleStore {
    pub items: RefCell<BTreeMap<String, RegionalSuspensionBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalSuspensionBundleStore for MemoryRegionalSuspensionBundleStore {
    fn find(&self, code: &str) -> Option<RegionalSuspensionBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalSuspensionBundle) {
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

    fn list(&self) -> Vec<RegionalSuspensionBundle> {
        let mut out: Vec<RegionalSuspensionBundle> = Vec::new();
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

pub fn new_memory_regional_suspension_bundle_store() -> Rc<dyn RegionalSuspensionBundleStore> {
    Rc::new(MemoryRegionalSuspensionBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalSuspensionBundleStore {
    pub inner: Rc<dyn RegionalSuspensionBundleStore>,
    pub cache: RefCell<cache::TtlCache<RegionalSuspensionBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalSuspensionBundleStore for CachedRegionalSuspensionBundleStore {
    fn find(&self, code: &str) -> Option<RegionalSuspensionBundle> {
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

    fn save(&self, item: &RegionalSuspensionBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalSuspensionBundle> {
        self.inner.list()
    }
}

pub fn new_cached_regional_suspension_bundle_store(
    inner: Rc<dyn RegionalSuspensionBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalSuspensionBundleStore> {
    Rc::new(CachedRegionalSuspensionBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalSuspensionBundle>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional suspension bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionBundleQuery {
    pub cluster: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalSuspensionBundleQuery {
    pub fn matches(&self, item: &RegionalSuspensionBundle) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalSuspensionBundle]) -> Vec<RegionalSuspensionBundle> {
        let matched = collections::filter(items, |candidate: &RegionalSuspensionBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_suspension_bundles(
    store: Rc<dyn RegionalSuspensionBundleStore>,
    query: &RegionalSuspensionBundleQuery,
) -> Vec<RegionalSuspensionBundle> {
    query.apply(&store.list())
}
