use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps data export rollups by code.
pub trait DataExportRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DataExportRollup>;
    fn save(&self, item: &DataExportRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DataExportRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDataExportRollupStore {
    pub items: RefCell<BTreeMap<String, DataExportRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl DataExportRollupStore for MemoryDataExportRollupStore {
    fn find(&self, code: &str) -> Option<DataExportRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DataExportRollup) {
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

    fn list(&self) -> Vec<DataExportRollup> {
        let mut out: Vec<DataExportRollup> = Vec::new();
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

pub fn new_memory_data_export_rollup_store() -> Rc<dyn DataExportRollupStore> {
    Rc::new(MemoryDataExportRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDataExportRollupStore {
    pub inner: Rc<dyn DataExportRollupStore>,
    pub cache: RefCell<cache::TtlCache<DataExportRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DataExportRollupStore for CachedDataExportRollupStore {
    fn find(&self, code: &str) -> Option<DataExportRollup> {
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

    fn save(&self, item: &DataExportRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DataExportRollup> {
        self.inner.list()
    }
}

pub fn new_cached_data_export_rollup_store(
    inner: Rc<dyn DataExportRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DataExportRollupStore> {
    Rc::new(CachedDataExportRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DataExportRollup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects data export rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRollupQuery {
    pub region: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DataExportRollupQuery {
    pub fn matches(&self, item: &DataExportRollup) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DataExportRollup]) -> Vec<DataExportRollup> {
        let matched = collections::filter(items, |candidate: &DataExportRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_data_export_rollups(
    store: Rc<dyn DataExportRollupStore>,
    query: &DataExportRollupQuery,
) -> Vec<DataExportRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DataExportRollupStore> = new_memory_data_export_rollup_store();
        let item = new_data_export_rollup("code-225", "title-823");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
