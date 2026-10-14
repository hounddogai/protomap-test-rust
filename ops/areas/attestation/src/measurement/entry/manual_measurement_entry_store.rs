use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual measurement entries by code.
pub trait ManualMeasurementEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualMeasurementEntry>;
    fn save(&self, item: &ManualMeasurementEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualMeasurementEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualMeasurementEntryStore {
    pub items: RefCell<BTreeMap<String, ManualMeasurementEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualMeasurementEntryStore for MemoryManualMeasurementEntryStore {
    fn find(&self, code: &str) -> Option<ManualMeasurementEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualMeasurementEntry) {
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

    fn list(&self) -> Vec<ManualMeasurementEntry> {
        let mut out: Vec<ManualMeasurementEntry> = Vec::new();
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

pub fn new_memory_manual_measurement_entry_store() -> Rc<dyn ManualMeasurementEntryStore> {
    Rc::new(MemoryManualMeasurementEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualMeasurementEntryStore {
    pub inner: Rc<dyn ManualMeasurementEntryStore>,
    pub cache: RefCell<cache::TtlCache<ManualMeasurementEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualMeasurementEntryStore for CachedManualMeasurementEntryStore {
    fn find(&self, code: &str) -> Option<ManualMeasurementEntry> {
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

    fn save(&self, item: &ManualMeasurementEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualMeasurementEntry> {
        self.inner.list()
    }
}

pub fn new_cached_manual_measurement_entry_store(
    inner: Rc<dyn ManualMeasurementEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualMeasurementEntryStore> {
    Rc::new(CachedManualMeasurementEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualMeasurementEntry>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual measurement entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementEntryQuery {
    pub source: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualMeasurementEntryQuery {
    pub fn matches(&self, item: &ManualMeasurementEntry) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualMeasurementEntry]) -> Vec<ManualMeasurementEntry> {
        let matched = collections::filter(items, |candidate: &ManualMeasurementEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_measurement_entries(
    store: Rc<dyn ManualMeasurementEntryStore>,
    query: &ManualMeasurementEntryQuery,
) -> Vec<ManualMeasurementEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualMeasurementEntryStore> = new_memory_manual_measurement_entry_store();
        let item = new_manual_measurement_entry("code-290", "title-444");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
