use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual data export groups by code.
pub trait ManualDataExportGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualDataExportGroup>;
    fn save(&self, item: &ManualDataExportGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualDataExportGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualDataExportGroupStore {
    pub items: RefCell<BTreeMap<String, ManualDataExportGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualDataExportGroupStore for MemoryManualDataExportGroupStore {
    fn find(&self, code: &str) -> Option<ManualDataExportGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualDataExportGroup) {
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

    fn list(&self) -> Vec<ManualDataExportGroup> {
        let mut out: Vec<ManualDataExportGroup> = Vec::new();
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

pub fn new_memory_manual_data_export_group_store() -> Rc<dyn ManualDataExportGroupStore> {
    Rc::new(MemoryManualDataExportGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualDataExportGroupStore {
    pub inner: Rc<dyn ManualDataExportGroupStore>,
    pub cache: RefCell<cache::TtlCache<ManualDataExportGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualDataExportGroupStore for CachedManualDataExportGroupStore {
    fn find(&self, code: &str) -> Option<ManualDataExportGroup> {
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

    fn save(&self, item: &ManualDataExportGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualDataExportGroup> {
        self.inner.list()
    }
}

pub fn new_cached_manual_data_export_group_store(
    inner: Rc<dyn ManualDataExportGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualDataExportGroupStore> {
    Rc::new(CachedManualDataExportGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualDataExportGroup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual data export groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportGroupQuery {
    pub region: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualDataExportGroupQuery {
    pub fn matches(&self, item: &ManualDataExportGroup) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualDataExportGroup]) -> Vec<ManualDataExportGroup> {
        let matched = collections::filter(items, |candidate: &ManualDataExportGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_data_export_groups(
    store: Rc<dyn ManualDataExportGroupStore>,
    query: &ManualDataExportGroupQuery,
) -> Vec<ManualDataExportGroup> {
    query.apply(&store.list())
}
