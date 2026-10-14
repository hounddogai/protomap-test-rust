use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps release manifests by code.
pub trait ReleaseManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReleaseManifest>;
    fn save(&self, item: &ReleaseManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReleaseManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReleaseManifestStore {
    pub items: RefCell<BTreeMap<String, ReleaseManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl ReleaseManifestStore for MemoryReleaseManifestStore {
    fn find(&self, code: &str) -> Option<ReleaseManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReleaseManifest) {
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

    fn list(&self) -> Vec<ReleaseManifest> {
        let mut out: Vec<ReleaseManifest> = Vec::new();
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

pub fn new_memory_release_manifest_store() -> Rc<dyn ReleaseManifestStore> {
    Rc::new(MemoryReleaseManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReleaseManifestStore {
    pub inner: Rc<dyn ReleaseManifestStore>,
    pub cache: RefCell<cache::TtlCache<ReleaseManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReleaseManifestStore for CachedReleaseManifestStore {
    fn find(&self, code: &str) -> Option<ReleaseManifest> {
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

    fn save(&self, item: &ReleaseManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReleaseManifest> {
        self.inner.list()
    }
}

pub fn new_cached_release_manifest_store(
    inner: Rc<dyn ReleaseManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReleaseManifestStore> {
    Rc::new(CachedReleaseManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReleaseManifest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects release manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseManifestQuery {
    pub name: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReleaseManifestQuery {
    pub fn matches(&self, item: &ReleaseManifest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReleaseManifest]) -> Vec<ReleaseManifest> {
        let matched = collections::filter(items, |candidate: &ReleaseManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_release_manifests(
    store: Rc<dyn ReleaseManifestStore>,
    query: &ReleaseManifestQuery,
) -> Vec<ReleaseManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReleaseManifestStore> = new_memory_release_manifest_store();
        let item = new_release_manifest("code-341", "name-679");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
