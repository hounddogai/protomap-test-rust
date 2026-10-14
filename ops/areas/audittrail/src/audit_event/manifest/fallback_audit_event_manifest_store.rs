use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback audit event manifests by code.
pub trait FallbackAuditEventManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackAuditEventManifest>;
    fn save(&self, item: &FallbackAuditEventManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackAuditEventManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackAuditEventManifestStore {
    pub items: RefCell<BTreeMap<String, FallbackAuditEventManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackAuditEventManifestStore for MemoryFallbackAuditEventManifestStore {
    fn find(&self, code: &str) -> Option<FallbackAuditEventManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackAuditEventManifest) {
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

    fn list(&self) -> Vec<FallbackAuditEventManifest> {
        let mut out: Vec<FallbackAuditEventManifest> = Vec::new();
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

pub fn new_memory_fallback_audit_event_manifest_store() -> Rc<dyn FallbackAuditEventManifestStore> {
    Rc::new(MemoryFallbackAuditEventManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackAuditEventManifestStore {
    pub inner: Rc<dyn FallbackAuditEventManifestStore>,
    pub cache: RefCell<cache::TtlCache<FallbackAuditEventManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackAuditEventManifestStore for CachedFallbackAuditEventManifestStore {
    fn find(&self, code: &str) -> Option<FallbackAuditEventManifest> {
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

    fn save(&self, item: &FallbackAuditEventManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackAuditEventManifest> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_audit_event_manifest_store(
    inner: Rc<dyn FallbackAuditEventManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackAuditEventManifestStore> {
    Rc::new(CachedFallbackAuditEventManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackAuditEventManifest>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback audit event manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventManifestQuery {
    pub cluster: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackAuditEventManifestQuery {
    pub fn matches(&self, item: &FallbackAuditEventManifest) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[FallbackAuditEventManifest]) -> Vec<FallbackAuditEventManifest> {
        let matched = collections::filter(items, |candidate: &FallbackAuditEventManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_audit_event_manifests(
    store: Rc<dyn FallbackAuditEventManifestStore>,
    query: &FallbackAuditEventManifestQuery,
) -> Vec<FallbackAuditEventManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackAuditEventManifestStore> = new_memory_fallback_audit_event_manifest_store();
        let item = new_fallback_audit_event_manifest("code-672", "title-623");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
