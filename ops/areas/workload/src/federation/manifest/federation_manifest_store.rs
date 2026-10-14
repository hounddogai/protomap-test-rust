use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps federation manifests by code.
pub trait FederationManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FederationManifest>;
    fn save(&self, item: &FederationManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FederationManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFederationManifestStore {
    pub items: RefCell<BTreeMap<String, FederationManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl FederationManifestStore for MemoryFederationManifestStore {
    fn find(&self, code: &str) -> Option<FederationManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FederationManifest) {
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

    fn list(&self) -> Vec<FederationManifest> {
        let mut out: Vec<FederationManifest> = Vec::new();
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

pub fn new_memory_federation_manifest_store() -> Rc<dyn FederationManifestStore> {
    Rc::new(MemoryFederationManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFederationManifestStore {
    pub inner: Rc<dyn FederationManifestStore>,
    pub cache: RefCell<cache::TtlCache<FederationManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FederationManifestStore for CachedFederationManifestStore {
    fn find(&self, code: &str) -> Option<FederationManifest> {
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

    fn save(&self, item: &FederationManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FederationManifest> {
        self.inner.list()
    }
}

pub fn new_cached_federation_manifest_store(
    inner: Rc<dyn FederationManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FederationManifestStore> {
    Rc::new(CachedFederationManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FederationManifest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects federation manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationManifestQuery {
    pub title: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FederationManifestQuery {
    pub fn matches(&self, item: &FederationManifest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[FederationManifest]) -> Vec<FederationManifest> {
        let matched = collections::filter(items, |candidate: &FederationManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_federation_manifests(
    store: Rc<dyn FederationManifestStore>,
    query: &FederationManifestQuery,
) -> Vec<FederationManifest> {
    query.apply(&store.list())
}
