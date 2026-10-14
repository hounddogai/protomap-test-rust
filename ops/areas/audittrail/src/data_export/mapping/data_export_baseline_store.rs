use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps data export baselines by code.
pub trait DataExportBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DataExportBaseline>;
    fn save(&self, item: &DataExportBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DataExportBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDataExportBaselineStore {
    pub items: RefCell<BTreeMap<String, DataExportBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl DataExportBaselineStore for MemoryDataExportBaselineStore {
    fn find(&self, code: &str) -> Option<DataExportBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DataExportBaseline) {
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

    fn list(&self) -> Vec<DataExportBaseline> {
        let mut out: Vec<DataExportBaseline> = Vec::new();
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

pub fn new_memory_data_export_baseline_store() -> Rc<dyn DataExportBaselineStore> {
    Rc::new(MemoryDataExportBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDataExportBaselineStore {
    pub inner: Rc<dyn DataExportBaselineStore>,
    pub cache: RefCell<cache::TtlCache<DataExportBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DataExportBaselineStore for CachedDataExportBaselineStore {
    fn find(&self, code: &str) -> Option<DataExportBaseline> {
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

    fn save(&self, item: &DataExportBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DataExportBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_data_export_baseline_store(
    inner: Rc<dyn DataExportBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DataExportBaselineStore> {
    Rc::new(CachedDataExportBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DataExportBaseline>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects data export baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportBaselineQuery {
    pub source: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DataExportBaselineQuery {
    pub fn matches(&self, item: &DataExportBaseline) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DataExportBaseline]) -> Vec<DataExportBaseline> {
        let matched = collections::filter(items, |candidate: &DataExportBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_data_export_baselines(
    store: Rc<dyn DataExportBaselineStore>,
    query: &DataExportBaselineQuery,
) -> Vec<DataExportBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DataExportBaselineStore> = new_memory_data_export_baseline_store();
        let item = new_data_export_baseline("code-80", "title-625");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
