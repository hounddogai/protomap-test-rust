use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps data export variances by code.
pub trait DataExportVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DataExportVariance>;
    fn save(&self, item: &DataExportVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DataExportVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDataExportVarianceStore {
    pub items: RefCell<BTreeMap<String, DataExportVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl DataExportVarianceStore for MemoryDataExportVarianceStore {
    fn find(&self, code: &str) -> Option<DataExportVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DataExportVariance) {
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

    fn list(&self) -> Vec<DataExportVariance> {
        let mut out: Vec<DataExportVariance> = Vec::new();
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

pub fn new_memory_data_export_variance_store() -> Rc<dyn DataExportVarianceStore> {
    Rc::new(MemoryDataExportVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDataExportVarianceStore {
    pub inner: Rc<dyn DataExportVarianceStore>,
    pub cache: RefCell<cache::TtlCache<DataExportVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DataExportVarianceStore for CachedDataExportVarianceStore {
    fn find(&self, code: &str) -> Option<DataExportVariance> {
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

    fn save(&self, item: &DataExportVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DataExportVariance> {
        self.inner.list()
    }
}

pub fn new_cached_data_export_variance_store(
    inner: Rc<dyn DataExportVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DataExportVarianceStore> {
    Rc::new(CachedDataExportVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DataExportVariance>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects data export variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportVarianceQuery {
    pub locale: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DataExportVarianceQuery {
    pub fn matches(&self, item: &DataExportVariance) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DataExportVariance]) -> Vec<DataExportVariance> {
        let matched = collections::filter(items, |candidate: &DataExportVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_data_export_variances(
    store: Rc<dyn DataExportVarianceStore>,
    query: &DataExportVarianceQuery,
) -> Vec<DataExportVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DataExportVarianceStore> = new_memory_data_export_variance_store();
        let item = new_data_export_variance("code-352", "label-188");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
