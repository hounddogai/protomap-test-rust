use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps data export records by code.
pub trait DataExportRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DataExportRecord>;
    fn save(&self, item: &DataExportRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DataExportRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDataExportRecordStore {
    pub items: RefCell<BTreeMap<String, DataExportRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl DataExportRecordStore for MemoryDataExportRecordStore {
    fn find(&self, code: &str) -> Option<DataExportRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DataExportRecord) {
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

    fn list(&self) -> Vec<DataExportRecord> {
        let mut out: Vec<DataExportRecord> = Vec::new();
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

pub fn new_memory_data_export_record_store() -> Rc<dyn DataExportRecordStore> {
    Rc::new(MemoryDataExportRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDataExportRecordStore {
    pub inner: Rc<dyn DataExportRecordStore>,
    pub cache: RefCell<cache::TtlCache<DataExportRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DataExportRecordStore for CachedDataExportRecordStore {
    fn find(&self, code: &str) -> Option<DataExportRecord> {
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

    fn save(&self, item: &DataExportRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DataExportRecord> {
        self.inner.list()
    }
}

pub fn new_cached_data_export_record_store(
    inner: Rc<dyn DataExportRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DataExportRecordStore> {
    Rc::new(CachedDataExportRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DataExportRecord>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects data export records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRecordQuery {
    pub category: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DataExportRecordQuery {
    pub fn matches(&self, item: &DataExportRecord) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DataExportRecord]) -> Vec<DataExportRecord> {
        let matched = collections::filter(items, |candidate: &DataExportRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_data_export_records(
    store: Rc<dyn DataExportRecordStore>,
    query: &DataExportRecordQuery,
) -> Vec<DataExportRecord> {
    query.apply(&store.list())
}
