use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary data export profiles by code.
pub trait PrimaryDataExportProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryDataExportProfile>;
    fn save(&self, item: &PrimaryDataExportProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryDataExportProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryDataExportProfileStore {
    pub items: RefCell<BTreeMap<String, PrimaryDataExportProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryDataExportProfileStore for MemoryPrimaryDataExportProfileStore {
    fn find(&self, code: &str) -> Option<PrimaryDataExportProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryDataExportProfile) {
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

    fn list(&self) -> Vec<PrimaryDataExportProfile> {
        let mut out: Vec<PrimaryDataExportProfile> = Vec::new();
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

pub fn new_memory_primary_data_export_profile_store() -> Rc<dyn PrimaryDataExportProfileStore> {
    Rc::new(MemoryPrimaryDataExportProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryDataExportProfileStore {
    pub inner: Rc<dyn PrimaryDataExportProfileStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryDataExportProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryDataExportProfileStore for CachedPrimaryDataExportProfileStore {
    fn find(&self, code: &str) -> Option<PrimaryDataExportProfile> {
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

    fn save(&self, item: &PrimaryDataExportProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryDataExportProfile> {
        self.inner.list()
    }
}

pub fn new_cached_primary_data_export_profile_store(
    inner: Rc<dyn PrimaryDataExportProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryDataExportProfileStore> {
    Rc::new(CachedPrimaryDataExportProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryDataExportProfile>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary data export profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportProfileQuery {
    pub namespace: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryDataExportProfileQuery {
    pub fn matches(&self, item: &PrimaryDataExportProfile) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[PrimaryDataExportProfile]) -> Vec<PrimaryDataExportProfile> {
        let matched = collections::filter(items, |candidate: &PrimaryDataExportProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_data_export_profiles(
    store: Rc<dyn PrimaryDataExportProfileStore>,
    query: &PrimaryDataExportProfileQuery,
) -> Vec<PrimaryDataExportProfile> {
    query.apply(&store.list())
}
