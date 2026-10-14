use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict measurement batches by code.
pub trait StrictMeasurementBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictMeasurementBatch>;
    fn save(&self, item: &StrictMeasurementBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictMeasurementBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictMeasurementBatchStore {
    pub items: RefCell<BTreeMap<String, StrictMeasurementBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictMeasurementBatchStore for MemoryStrictMeasurementBatchStore {
    fn find(&self, code: &str) -> Option<StrictMeasurementBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictMeasurementBatch) {
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

    fn list(&self) -> Vec<StrictMeasurementBatch> {
        let mut out: Vec<StrictMeasurementBatch> = Vec::new();
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

pub fn new_memory_strict_measurement_batch_store() -> Rc<dyn StrictMeasurementBatchStore> {
    Rc::new(MemoryStrictMeasurementBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictMeasurementBatchStore {
    pub inner: Rc<dyn StrictMeasurementBatchStore>,
    pub cache: RefCell<cache::TtlCache<StrictMeasurementBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictMeasurementBatchStore for CachedStrictMeasurementBatchStore {
    fn find(&self, code: &str) -> Option<StrictMeasurementBatch> {
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

    fn save(&self, item: &StrictMeasurementBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictMeasurementBatch> {
        self.inner.list()
    }
}

pub fn new_cached_strict_measurement_batch_store(
    inner: Rc<dyn StrictMeasurementBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictMeasurementBatchStore> {
    Rc::new(CachedStrictMeasurementBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictMeasurementBatch>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict measurement batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictMeasurementBatchQuery {
    pub title: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictMeasurementBatchQuery {
    pub fn matches(&self, item: &StrictMeasurementBatch) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictMeasurementBatch]) -> Vec<StrictMeasurementBatch> {
        let matched = collections::filter(items, |candidate: &StrictMeasurementBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_measurement_batches(
    store: Rc<dyn StrictMeasurementBatchStore>,
    query: &StrictMeasurementBatchQuery,
) -> Vec<StrictMeasurementBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictMeasurementBatchStore> = new_memory_strict_measurement_batch_store();
        let item = new_strict_measurement_batch("code-739", "title-851");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
