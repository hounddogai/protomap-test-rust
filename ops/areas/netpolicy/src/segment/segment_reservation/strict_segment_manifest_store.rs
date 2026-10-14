use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict segment manifests by code.
pub trait StrictSegmentManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictSegmentManifest>;
    fn save(&self, item: &StrictSegmentManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictSegmentManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictSegmentManifestStore {
    pub items: RefCell<BTreeMap<String, StrictSegmentManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictSegmentManifestStore for MemoryStrictSegmentManifestStore {
    fn find(&self, code: &str) -> Option<StrictSegmentManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictSegmentManifest) {
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

    fn list(&self) -> Vec<StrictSegmentManifest> {
        let mut out: Vec<StrictSegmentManifest> = Vec::new();
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

pub fn new_memory_strict_segment_manifest_store() -> Rc<dyn StrictSegmentManifestStore> {
    Rc::new(MemoryStrictSegmentManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictSegmentManifestStore {
    pub inner: Rc<dyn StrictSegmentManifestStore>,
    pub cache: RefCell<cache::TtlCache<StrictSegmentManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictSegmentManifestStore for CachedStrictSegmentManifestStore {
    fn find(&self, code: &str) -> Option<StrictSegmentManifest> {
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

    fn save(&self, item: &StrictSegmentManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictSegmentManifest> {
        self.inner.list()
    }
}

pub fn new_cached_strict_segment_manifest_store(
    inner: Rc<dyn StrictSegmentManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictSegmentManifestStore> {
    Rc::new(CachedStrictSegmentManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictSegmentManifest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict segment manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentManifestQuery {
    pub source: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictSegmentManifestQuery {
    pub fn matches(&self, item: &StrictSegmentManifest) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictSegmentManifest]) -> Vec<StrictSegmentManifest> {
        let matched = collections::filter(items, |candidate: &StrictSegmentManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_segment_manifests(
    store: Rc<dyn StrictSegmentManifestStore>,
    query: &StrictSegmentManifestQuery,
) -> Vec<StrictSegmentManifest> {
    query.apply(&store.list())
}
