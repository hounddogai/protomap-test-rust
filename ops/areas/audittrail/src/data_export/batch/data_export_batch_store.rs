use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps data export batches by code.
pub trait DataExportBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DataExportBatch>;
    fn save(&self, item: &DataExportBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DataExportBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDataExportBatchStore {
    pub items: RefCell<BTreeMap<String, DataExportBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl DataExportBatchStore for MemoryDataExportBatchStore {
    fn find(&self, code: &str) -> Option<DataExportBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DataExportBatch) {
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

    fn list(&self) -> Vec<DataExportBatch> {
        let mut out: Vec<DataExportBatch> = Vec::new();
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

pub fn new_memory_data_export_batch_store() -> Rc<dyn DataExportBatchStore> {
    Rc::new(MemoryDataExportBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDataExportBatchStore {
    pub inner: Rc<dyn DataExportBatchStore>,
    pub cache: RefCell<cache::TtlCache<DataExportBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DataExportBatchStore for CachedDataExportBatchStore {
    fn find(&self, code: &str) -> Option<DataExportBatch> {
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

    fn save(&self, item: &DataExportBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DataExportBatch> {
        self.inner.list()
    }
}

pub fn new_cached_data_export_batch_store(
    inner: Rc<dyn DataExportBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DataExportBatchStore> {
    Rc::new(CachedDataExportBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DataExportBatch>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects data export batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportBatchQuery {
    pub label: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DataExportBatchQuery {
    pub fn matches(&self, item: &DataExportBatch) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DataExportBatch]) -> Vec<DataExportBatch> {
        let matched = collections::filter(items, |candidate: &DataExportBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_data_export_batches(
    store: Rc<dyn DataExportBatchStore>,
    query: &DataExportBatchQuery,
) -> Vec<DataExportBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DataExportBatchStore> = new_memory_data_export_batch_store();
        let item = new_data_export_batch("code-260", "label-509");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
