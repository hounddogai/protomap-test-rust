use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted alert batches by code.
pub trait WeightedAlertBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedAlertBatch>;
    fn save(&self, item: &WeightedAlertBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedAlertBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedAlertBatchStore {
    pub items: RefCell<BTreeMap<String, WeightedAlertBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedAlertBatchStore for MemoryWeightedAlertBatchStore {
    fn find(&self, code: &str) -> Option<WeightedAlertBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedAlertBatch) {
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

    fn list(&self) -> Vec<WeightedAlertBatch> {
        let mut out: Vec<WeightedAlertBatch> = Vec::new();
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

pub fn new_memory_weighted_alert_batch_store() -> Rc<dyn WeightedAlertBatchStore> {
    Rc::new(MemoryWeightedAlertBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedAlertBatchStore {
    pub inner: Rc<dyn WeightedAlertBatchStore>,
    pub cache: RefCell<cache::TtlCache<WeightedAlertBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedAlertBatchStore for CachedWeightedAlertBatchStore {
    fn find(&self, code: &str) -> Option<WeightedAlertBatch> {
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

    fn save(&self, item: &WeightedAlertBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedAlertBatch> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_alert_batch_store(
    inner: Rc<dyn WeightedAlertBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedAlertBatchStore> {
    Rc::new(CachedWeightedAlertBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedAlertBatch>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted alert batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertBatchQuery {
    pub country: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedAlertBatchQuery {
    pub fn matches(&self, item: &WeightedAlertBatch) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedAlertBatch]) -> Vec<WeightedAlertBatch> {
        let matched = collections::filter(items, |candidate: &WeightedAlertBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_alert_batches(
    store: Rc<dyn WeightedAlertBatchStore>,
    query: &WeightedAlertBatchQuery,
) -> Vec<WeightedAlertBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedAlertBatchStore> = new_memory_weighted_alert_batch_store();
        let item = new_weighted_alert_batch("code-459", "label-373");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
