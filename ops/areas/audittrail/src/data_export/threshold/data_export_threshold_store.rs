use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps data export thresholds by code.
pub trait DataExportThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DataExportThreshold>;
    fn save(&self, item: &DataExportThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DataExportThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDataExportThresholdStore {
    pub items: RefCell<BTreeMap<String, DataExportThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl DataExportThresholdStore for MemoryDataExportThresholdStore {
    fn find(&self, code: &str) -> Option<DataExportThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DataExportThreshold) {
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

    fn list(&self) -> Vec<DataExportThreshold> {
        let mut out: Vec<DataExportThreshold> = Vec::new();
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

pub fn new_memory_data_export_threshold_store() -> Rc<dyn DataExportThresholdStore> {
    Rc::new(MemoryDataExportThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDataExportThresholdStore {
    pub inner: Rc<dyn DataExportThresholdStore>,
    pub cache: RefCell<cache::TtlCache<DataExportThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DataExportThresholdStore for CachedDataExportThresholdStore {
    fn find(&self, code: &str) -> Option<DataExportThreshold> {
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

    fn save(&self, item: &DataExportThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DataExportThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_data_export_threshold_store(
    inner: Rc<dyn DataExportThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DataExportThresholdStore> {
    Rc::new(CachedDataExportThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DataExportThreshold>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects data export thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportThresholdQuery {
    pub label: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DataExportThresholdQuery {
    pub fn matches(&self, item: &DataExportThreshold) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DataExportThreshold]) -> Vec<DataExportThreshold> {
        let matched = collections::filter(items, |candidate: &DataExportThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_data_export_thresholds(
    store: Rc<dyn DataExportThresholdStore>,
    query: &DataExportThresholdQuery,
) -> Vec<DataExportThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DataExportThresholdStore> = new_memory_data_export_threshold_store();
        let item = new_data_export_threshold("code-316", "label-543");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
