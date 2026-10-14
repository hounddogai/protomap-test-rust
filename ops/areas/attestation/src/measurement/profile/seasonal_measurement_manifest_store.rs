use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal measurement manifests by code.
pub trait SeasonalMeasurementManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalMeasurementManifest>;
    fn save(&self, item: &SeasonalMeasurementManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalMeasurementManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalMeasurementManifestStore {
    pub items: RefCell<BTreeMap<String, SeasonalMeasurementManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalMeasurementManifestStore for MemorySeasonalMeasurementManifestStore {
    fn find(&self, code: &str) -> Option<SeasonalMeasurementManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalMeasurementManifest) {
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

    fn list(&self) -> Vec<SeasonalMeasurementManifest> {
        let mut out: Vec<SeasonalMeasurementManifest> = Vec::new();
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

pub fn new_memory_seasonal_measurement_manifest_store() -> Rc<dyn SeasonalMeasurementManifestStore> {
    Rc::new(MemorySeasonalMeasurementManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalMeasurementManifestStore {
    pub inner: Rc<dyn SeasonalMeasurementManifestStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalMeasurementManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalMeasurementManifestStore for CachedSeasonalMeasurementManifestStore {
    fn find(&self, code: &str) -> Option<SeasonalMeasurementManifest> {
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

    fn save(&self, item: &SeasonalMeasurementManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalMeasurementManifest> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_measurement_manifest_store(
    inner: Rc<dyn SeasonalMeasurementManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalMeasurementManifestStore> {
    Rc::new(CachedSeasonalMeasurementManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalMeasurementManifest>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal measurement manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMeasurementManifestQuery {
    pub title: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalMeasurementManifestQuery {
    pub fn matches(&self, item: &SeasonalMeasurementManifest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalMeasurementManifest]) -> Vec<SeasonalMeasurementManifest> {
        let matched = collections::filter(items, |candidate: &SeasonalMeasurementManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_measurement_manifests(
    store: Rc<dyn SeasonalMeasurementManifestStore>,
    query: &SeasonalMeasurementManifestQuery,
) -> Vec<SeasonalMeasurementManifest> {
    query.apply(&store.list())
}
