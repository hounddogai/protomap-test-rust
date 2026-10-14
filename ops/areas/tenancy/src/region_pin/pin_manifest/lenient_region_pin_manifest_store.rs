use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient region pin manifests by code.
pub trait LenientRegionPinManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientRegionPinManifest>;
    fn save(&self, item: &LenientRegionPinManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientRegionPinManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientRegionPinManifestStore {
    pub items: RefCell<BTreeMap<String, LenientRegionPinManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientRegionPinManifestStore for MemoryLenientRegionPinManifestStore {
    fn find(&self, code: &str) -> Option<LenientRegionPinManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientRegionPinManifest) {
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

    fn list(&self) -> Vec<LenientRegionPinManifest> {
        let mut out: Vec<LenientRegionPinManifest> = Vec::new();
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

pub fn new_memory_lenient_region_pin_manifest_store() -> Rc<dyn LenientRegionPinManifestStore> {
    Rc::new(MemoryLenientRegionPinManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientRegionPinManifestStore {
    pub inner: Rc<dyn LenientRegionPinManifestStore>,
    pub cache: RefCell<cache::TtlCache<LenientRegionPinManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientRegionPinManifestStore for CachedLenientRegionPinManifestStore {
    fn find(&self, code: &str) -> Option<LenientRegionPinManifest> {
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

    fn save(&self, item: &LenientRegionPinManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientRegionPinManifest> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_region_pin_manifest_store(
    inner: Rc<dyn LenientRegionPinManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientRegionPinManifestStore> {
    Rc::new(CachedLenientRegionPinManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientRegionPinManifest>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient region pin manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinManifestQuery {
    pub scope: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientRegionPinManifestQuery {
    pub fn matches(&self, item: &LenientRegionPinManifest) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[LenientRegionPinManifest]) -> Vec<LenientRegionPinManifest> {
        let matched = collections::filter(items, |candidate: &LenientRegionPinManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_region_pin_manifests(
    store: Rc<dyn LenientRegionPinManifestStore>,
    query: &LenientRegionPinManifestQuery,
) -> Vec<LenientRegionPinManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientRegionPinManifestStore> = new_memory_lenient_region_pin_manifest_store();
        let item = new_lenient_region_pin_manifest("code-810", "label-679");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
