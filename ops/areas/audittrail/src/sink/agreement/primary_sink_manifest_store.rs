use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary sink manifests by code.
pub trait PrimarySinkManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimarySinkManifest>;
    fn save(&self, item: &PrimarySinkManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimarySinkManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimarySinkManifestStore {
    pub items: RefCell<BTreeMap<String, PrimarySinkManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimarySinkManifestStore for MemoryPrimarySinkManifestStore {
    fn find(&self, code: &str) -> Option<PrimarySinkManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimarySinkManifest) {
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

    fn list(&self) -> Vec<PrimarySinkManifest> {
        let mut out: Vec<PrimarySinkManifest> = Vec::new();
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

pub fn new_memory_primary_sink_manifest_store() -> Rc<dyn PrimarySinkManifestStore> {
    Rc::new(MemoryPrimarySinkManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimarySinkManifestStore {
    pub inner: Rc<dyn PrimarySinkManifestStore>,
    pub cache: RefCell<cache::TtlCache<PrimarySinkManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimarySinkManifestStore for CachedPrimarySinkManifestStore {
    fn find(&self, code: &str) -> Option<PrimarySinkManifest> {
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

    fn save(&self, item: &PrimarySinkManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimarySinkManifest> {
        self.inner.list()
    }
}

pub fn new_cached_primary_sink_manifest_store(
    inner: Rc<dyn PrimarySinkManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimarySinkManifestStore> {
    Rc::new(CachedPrimarySinkManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimarySinkManifest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary sink manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkManifestQuery {
    pub namespace: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimarySinkManifestQuery {
    pub fn matches(&self, item: &PrimarySinkManifest) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[PrimarySinkManifest]) -> Vec<PrimarySinkManifest> {
        let matched = collections::filter(items, |candidate: &PrimarySinkManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_sink_manifests(
    store: Rc<dyn PrimarySinkManifestStore>,
    query: &PrimarySinkManifestQuery,
) -> Vec<PrimarySinkManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimarySinkManifestStore> = new_memory_primary_sink_manifest_store();
        let item = new_primary_sink_manifest("code-891", "label-778");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
