use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps evidence manifests by code.
pub trait EvidenceManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EvidenceManifest>;
    fn save(&self, item: &EvidenceManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EvidenceManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEvidenceManifestStore {
    pub items: RefCell<BTreeMap<String, EvidenceManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl EvidenceManifestStore for MemoryEvidenceManifestStore {
    fn find(&self, code: &str) -> Option<EvidenceManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EvidenceManifest) {
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

    fn list(&self) -> Vec<EvidenceManifest> {
        let mut out: Vec<EvidenceManifest> = Vec::new();
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

pub fn new_memory_evidence_manifest_store() -> Rc<dyn EvidenceManifestStore> {
    Rc::new(MemoryEvidenceManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEvidenceManifestStore {
    pub inner: Rc<dyn EvidenceManifestStore>,
    pub cache: RefCell<cache::TtlCache<EvidenceManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EvidenceManifestStore for CachedEvidenceManifestStore {
    fn find(&self, code: &str) -> Option<EvidenceManifest> {
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

    fn save(&self, item: &EvidenceManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EvidenceManifest> {
        self.inner.list()
    }
}

pub fn new_cached_evidence_manifest_store(
    inner: Rc<dyn EvidenceManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EvidenceManifestStore> {
    Rc::new(CachedEvidenceManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EvidenceManifest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects evidence manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceManifestQuery {
    pub label: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EvidenceManifestQuery {
    pub fn matches(&self, item: &EvidenceManifest) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EvidenceManifest]) -> Vec<EvidenceManifest> {
        let matched = collections::filter(items, |candidate: &EvidenceManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_evidence_manifests(
    store: Rc<dyn EvidenceManifestStore>,
    query: &EvidenceManifestQuery,
) -> Vec<EvidenceManifest> {
    query.apply(&store.list())
}
