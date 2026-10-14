use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps canary manifests by code.
pub trait CanaryManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CanaryManifest>;
    fn save(&self, item: &CanaryManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CanaryManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCanaryManifestStore {
    pub items: RefCell<BTreeMap<String, CanaryManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl CanaryManifestStore for MemoryCanaryManifestStore {
    fn find(&self, code: &str) -> Option<CanaryManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CanaryManifest) {
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

    fn list(&self) -> Vec<CanaryManifest> {
        let mut out: Vec<CanaryManifest> = Vec::new();
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

pub fn new_memory_canary_manifest_store() -> Rc<dyn CanaryManifestStore> {
    Rc::new(MemoryCanaryManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCanaryManifestStore {
    pub inner: Rc<dyn CanaryManifestStore>,
    pub cache: RefCell<cache::TtlCache<CanaryManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CanaryManifestStore for CachedCanaryManifestStore {
    fn find(&self, code: &str) -> Option<CanaryManifest> {
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

    fn save(&self, item: &CanaryManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CanaryManifest> {
        self.inner.list()
    }
}

pub fn new_cached_canary_manifest_store(
    inner: Rc<dyn CanaryManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CanaryManifestStore> {
    Rc::new(CachedCanaryManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CanaryManifest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects canary manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryManifestQuery {
    pub name: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CanaryManifestQuery {
    pub fn matches(&self, item: &CanaryManifest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CanaryManifest]) -> Vec<CanaryManifest> {
        let matched = collections::filter(items, |candidate: &CanaryManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_canary_manifests(store: Rc<dyn CanaryManifestStore>, query: &CanaryManifestQuery) -> Vec<CanaryManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CanaryManifestStore> = new_memory_canary_manifest_store();
        let item = new_canary_manifest("code-607", "name-695");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
