use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending nonce manifests by code.
pub trait PendingNonceManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingNonceManifest>;
    fn save(&self, item: &PendingNonceManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingNonceManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingNonceManifestStore {
    pub items: RefCell<BTreeMap<String, PendingNonceManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingNonceManifestStore for MemoryPendingNonceManifestStore {
    fn find(&self, code: &str) -> Option<PendingNonceManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingNonceManifest) {
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

    fn list(&self) -> Vec<PendingNonceManifest> {
        let mut out: Vec<PendingNonceManifest> = Vec::new();
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

pub fn new_memory_pending_nonce_manifest_store() -> Rc<dyn PendingNonceManifestStore> {
    Rc::new(MemoryPendingNonceManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingNonceManifestStore {
    pub inner: Rc<dyn PendingNonceManifestStore>,
    pub cache: RefCell<cache::TtlCache<PendingNonceManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingNonceManifestStore for CachedPendingNonceManifestStore {
    fn find(&self, code: &str) -> Option<PendingNonceManifest> {
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

    fn save(&self, item: &PendingNonceManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingNonceManifest> {
        self.inner.list()
    }
}

pub fn new_cached_pending_nonce_manifest_store(
    inner: Rc<dyn PendingNonceManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingNonceManifestStore> {
    Rc::new(CachedPendingNonceManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingNonceManifest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending nonce manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceManifestQuery {
    pub locale: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingNonceManifestQuery {
    pub fn matches(&self, item: &PendingNonceManifest) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingNonceManifest]) -> Vec<PendingNonceManifest> {
        let matched = collections::filter(items, |candidate: &PendingNonceManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_nonce_manifests(
    store: Rc<dyn PendingNonceManifestStore>,
    query: &PendingNonceManifestQuery,
) -> Vec<PendingNonceManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingNonceManifestStore> = new_memory_pending_nonce_manifest_store();
        let item = new_pending_nonce_manifest("code-329", "label-608");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
