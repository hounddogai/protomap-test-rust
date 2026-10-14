use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary region pin manifests by code.
pub trait PrimaryRegionPinManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryRegionPinManifest>;
    fn save(&self, item: &PrimaryRegionPinManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryRegionPinManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryRegionPinManifestStore {
    pub items: RefCell<BTreeMap<String, PrimaryRegionPinManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryRegionPinManifestStore for MemoryPrimaryRegionPinManifestStore {
    fn find(&self, code: &str) -> Option<PrimaryRegionPinManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryRegionPinManifest) {
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

    fn list(&self) -> Vec<PrimaryRegionPinManifest> {
        let mut out: Vec<PrimaryRegionPinManifest> = Vec::new();
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

pub fn new_memory_primary_region_pin_manifest_store() -> Rc<dyn PrimaryRegionPinManifestStore> {
    Rc::new(MemoryPrimaryRegionPinManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryRegionPinManifestStore {
    pub inner: Rc<dyn PrimaryRegionPinManifestStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryRegionPinManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryRegionPinManifestStore for CachedPrimaryRegionPinManifestStore {
    fn find(&self, code: &str) -> Option<PrimaryRegionPinManifest> {
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

    fn save(&self, item: &PrimaryRegionPinManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryRegionPinManifest> {
        self.inner.list()
    }
}

pub fn new_cached_primary_region_pin_manifest_store(
    inner: Rc<dyn PrimaryRegionPinManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryRegionPinManifestStore> {
    Rc::new(CachedPrimaryRegionPinManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryRegionPinManifest>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary region pin manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinManifestQuery {
    pub name: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryRegionPinManifestQuery {
    pub fn matches(&self, item: &PrimaryRegionPinManifest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryRegionPinManifest]) -> Vec<PrimaryRegionPinManifest> {
        let matched = collections::filter(items, |candidate: &PrimaryRegionPinManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_region_pin_manifests(
    store: Rc<dyn PrimaryRegionPinManifestStore>,
    query: &PrimaryRegionPinManifestQuery,
) -> Vec<PrimaryRegionPinManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryRegionPinManifestStore> = new_memory_primary_region_pin_manifest_store();
        let item = new_primary_region_pin_manifest("code-70", "name-363");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
