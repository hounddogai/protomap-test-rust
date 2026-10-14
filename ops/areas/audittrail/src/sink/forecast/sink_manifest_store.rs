use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink manifests by code.
pub trait SinkManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkManifest>;
    fn save(&self, item: &SinkManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkManifestStore {
    pub items: RefCell<BTreeMap<String, SinkManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkManifestStore for MemorySinkManifestStore {
    fn find(&self, code: &str) -> Option<SinkManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkManifest) {
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

    fn list(&self) -> Vec<SinkManifest> {
        let mut out: Vec<SinkManifest> = Vec::new();
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

pub fn new_memory_sink_manifest_store() -> Rc<dyn SinkManifestStore> {
    Rc::new(MemorySinkManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkManifestStore {
    pub inner: Rc<dyn SinkManifestStore>,
    pub cache: RefCell<cache::TtlCache<SinkManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkManifestStore for CachedSinkManifestStore {
    fn find(&self, code: &str) -> Option<SinkManifest> {
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

    fn save(&self, item: &SinkManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkManifest> {
        self.inner.list()
    }
}

pub fn new_cached_sink_manifest_store(
    inner: Rc<dyn SinkManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkManifestStore> {
    Rc::new(CachedSinkManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkManifest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkManifestQuery {
    pub source: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkManifestQuery {
    pub fn matches(&self, item: &SinkManifest) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SinkManifest]) -> Vec<SinkManifest> {
        let matched = collections::filter(items, |candidate: &SinkManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_manifests(store: Rc<dyn SinkManifestStore>, query: &SinkManifestQuery) -> Vec<SinkManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SinkManifestStore> = new_memory_sink_manifest_store();
        let item = new_sink_manifest("code-191", "label-453");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
