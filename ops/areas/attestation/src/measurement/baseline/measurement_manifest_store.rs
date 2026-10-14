use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement manifests by code.
pub trait MeasurementManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementManifest>;
    fn save(&self, item: &MeasurementManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementManifestStore {
    pub items: RefCell<BTreeMap<String, MeasurementManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementManifestStore for MemoryMeasurementManifestStore {
    fn find(&self, code: &str) -> Option<MeasurementManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementManifest) {
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

    fn list(&self) -> Vec<MeasurementManifest> {
        let mut out: Vec<MeasurementManifest> = Vec::new();
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

pub fn new_memory_measurement_manifest_store() -> Rc<dyn MeasurementManifestStore> {
    Rc::new(MemoryMeasurementManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementManifestStore {
    pub inner: Rc<dyn MeasurementManifestStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementManifestStore for CachedMeasurementManifestStore {
    fn find(&self, code: &str) -> Option<MeasurementManifest> {
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

    fn save(&self, item: &MeasurementManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementManifest> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_manifest_store(
    inner: Rc<dyn MeasurementManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementManifestStore> {
    Rc::new(CachedMeasurementManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementManifest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementManifestQuery {
    pub unit: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementManifestQuery {
    pub fn matches(&self, item: &MeasurementManifest) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[MeasurementManifest]) -> Vec<MeasurementManifest> {
        let matched = collections::filter(items, |candidate: &MeasurementManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_manifests(
    store: Rc<dyn MeasurementManifestStore>,
    query: &MeasurementManifestQuery,
) -> Vec<MeasurementManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MeasurementManifestStore> = new_memory_measurement_manifest_store();
        let item = new_measurement_manifest("code-754", "title-466");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
