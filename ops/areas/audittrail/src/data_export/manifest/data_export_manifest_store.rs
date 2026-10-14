use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps data export manifests by code.
pub trait DataExportManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DataExportManifest>;
    fn save(&self, item: &DataExportManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DataExportManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDataExportManifestStore {
    pub items: RefCell<BTreeMap<String, DataExportManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl DataExportManifestStore for MemoryDataExportManifestStore {
    fn find(&self, code: &str) -> Option<DataExportManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DataExportManifest) {
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

    fn list(&self) -> Vec<DataExportManifest> {
        let mut out: Vec<DataExportManifest> = Vec::new();
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

pub fn new_memory_data_export_manifest_store() -> Rc<dyn DataExportManifestStore> {
    Rc::new(MemoryDataExportManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDataExportManifestStore {
    pub inner: Rc<dyn DataExportManifestStore>,
    pub cache: RefCell<cache::TtlCache<DataExportManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DataExportManifestStore for CachedDataExportManifestStore {
    fn find(&self, code: &str) -> Option<DataExportManifest> {
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

    fn save(&self, item: &DataExportManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DataExportManifest> {
        self.inner.list()
    }
}

pub fn new_cached_data_export_manifest_store(
    inner: Rc<dyn DataExportManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DataExportManifestStore> {
    Rc::new(CachedDataExportManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DataExportManifest>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects data export manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportManifestQuery {
    pub cluster: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DataExportManifestQuery {
    pub fn matches(&self, item: &DataExportManifest) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DataExportManifest]) -> Vec<DataExportManifest> {
        let matched = collections::filter(items, |candidate: &DataExportManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_data_export_manifests(
    store: Rc<dyn DataExportManifestStore>,
    query: &DataExportManifestQuery,
) -> Vec<DataExportManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DataExportManifestStore> = new_memory_data_export_manifest_store();
        let item = new_data_export_manifest("code-555", "name-539");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
