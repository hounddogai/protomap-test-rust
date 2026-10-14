use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual data export windows by code.
pub trait ManualDataExportWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualDataExportWindow>;
    fn save(&self, item: &ManualDataExportWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualDataExportWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualDataExportWindowStore {
    pub items: RefCell<BTreeMap<String, ManualDataExportWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualDataExportWindowStore for MemoryManualDataExportWindowStore {
    fn find(&self, code: &str) -> Option<ManualDataExportWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualDataExportWindow) {
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

    fn list(&self) -> Vec<ManualDataExportWindow> {
        let mut out: Vec<ManualDataExportWindow> = Vec::new();
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

pub fn new_memory_manual_data_export_window_store() -> Rc<dyn ManualDataExportWindowStore> {
    Rc::new(MemoryManualDataExportWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualDataExportWindowStore {
    pub inner: Rc<dyn ManualDataExportWindowStore>,
    pub cache: RefCell<cache::TtlCache<ManualDataExportWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualDataExportWindowStore for CachedManualDataExportWindowStore {
    fn find(&self, code: &str) -> Option<ManualDataExportWindow> {
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

    fn save(&self, item: &ManualDataExportWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualDataExportWindow> {
        self.inner.list()
    }
}

pub fn new_cached_manual_data_export_window_store(
    inner: Rc<dyn ManualDataExportWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualDataExportWindowStore> {
    Rc::new(CachedManualDataExportWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualDataExportWindow>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual data export windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportWindowQuery {
    pub title: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualDataExportWindowQuery {
    pub fn matches(&self, item: &ManualDataExportWindow) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualDataExportWindow]) -> Vec<ManualDataExportWindow> {
        let matched = collections::filter(items, |candidate: &ManualDataExportWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_data_export_windows(
    store: Rc<dyn ManualDataExportWindowStore>,
    query: &ManualDataExportWindowQuery,
) -> Vec<ManualDataExportWindow> {
    query.apply(&store.list())
}
