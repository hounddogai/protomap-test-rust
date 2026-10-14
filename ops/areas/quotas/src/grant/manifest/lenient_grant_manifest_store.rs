use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient grant manifests by code.
pub trait LenientGrantManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientGrantManifest>;
    fn save(&self, item: &LenientGrantManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientGrantManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientGrantManifestStore {
    pub items: RefCell<BTreeMap<String, LenientGrantManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientGrantManifestStore for MemoryLenientGrantManifestStore {
    fn find(&self, code: &str) -> Option<LenientGrantManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientGrantManifest) {
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

    fn list(&self) -> Vec<LenientGrantManifest> {
        let mut out: Vec<LenientGrantManifest> = Vec::new();
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

pub fn new_memory_lenient_grant_manifest_store() -> Rc<dyn LenientGrantManifestStore> {
    Rc::new(MemoryLenientGrantManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientGrantManifestStore {
    pub inner: Rc<dyn LenientGrantManifestStore>,
    pub cache: RefCell<cache::TtlCache<LenientGrantManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientGrantManifestStore for CachedLenientGrantManifestStore {
    fn find(&self, code: &str) -> Option<LenientGrantManifest> {
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

    fn save(&self, item: &LenientGrantManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientGrantManifest> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_grant_manifest_store(
    inner: Rc<dyn LenientGrantManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientGrantManifestStore> {
    Rc::new(CachedLenientGrantManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientGrantManifest>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient grant manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantManifestQuery {
    pub namespace: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientGrantManifestQuery {
    pub fn matches(&self, item: &LenientGrantManifest) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientGrantManifest]) -> Vec<LenientGrantManifest> {
        let matched = collections::filter(items, |candidate: &LenientGrantManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_grant_manifests(
    store: Rc<dyn LenientGrantManifestStore>,
    query: &LenientGrantManifestQuery,
) -> Vec<LenientGrantManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientGrantManifestStore> = new_memory_lenient_grant_manifest_store();
        let item = new_lenient_grant_manifest("code-201", "name-55");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
