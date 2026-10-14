use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps policy manifests by code.
pub trait PolicyManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PolicyManifest>;
    fn save(&self, item: &PolicyManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PolicyManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPolicyManifestStore {
    pub items: RefCell<BTreeMap<String, PolicyManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl PolicyManifestStore for MemoryPolicyManifestStore {
    fn find(&self, code: &str) -> Option<PolicyManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PolicyManifest) {
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

    fn list(&self) -> Vec<PolicyManifest> {
        let mut out: Vec<PolicyManifest> = Vec::new();
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

pub fn new_memory_policy_manifest_store() -> Rc<dyn PolicyManifestStore> {
    Rc::new(MemoryPolicyManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPolicyManifestStore {
    pub inner: Rc<dyn PolicyManifestStore>,
    pub cache: RefCell<cache::TtlCache<PolicyManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PolicyManifestStore for CachedPolicyManifestStore {
    fn find(&self, code: &str) -> Option<PolicyManifest> {
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

    fn save(&self, item: &PolicyManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PolicyManifest> {
        self.inner.list()
    }
}

pub fn new_cached_policy_manifest_store(
    inner: Rc<dyn PolicyManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PolicyManifestStore> {
    Rc::new(CachedPolicyManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PolicyManifest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects policy manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyManifestQuery {
    pub label: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PolicyManifestQuery {
    pub fn matches(&self, item: &PolicyManifest) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PolicyManifest]) -> Vec<PolicyManifest> {
        let matched = collections::filter(items, |candidate: &PolicyManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_policy_manifests(store: Rc<dyn PolicyManifestStore>, query: &PolicyManifestQuery) -> Vec<PolicyManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PolicyManifestStore> = new_memory_policy_manifest_store();
        let item = new_policy_manifest("code-900", "label-528");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
