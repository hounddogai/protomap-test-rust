use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement estimates by code.
pub trait MeasurementEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementEstimate>;
    fn save(&self, item: &MeasurementEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementEstimateStore {
    pub items: RefCell<BTreeMap<String, MeasurementEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementEstimateStore for MemoryMeasurementEstimateStore {
    fn find(&self, code: &str) -> Option<MeasurementEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementEstimate) {
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

    fn list(&self) -> Vec<MeasurementEstimate> {
        let mut out: Vec<MeasurementEstimate> = Vec::new();
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

pub fn new_memory_measurement_estimate_store() -> Rc<dyn MeasurementEstimateStore> {
    Rc::new(MemoryMeasurementEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementEstimateStore {
    pub inner: Rc<dyn MeasurementEstimateStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementEstimateStore for CachedMeasurementEstimateStore {
    fn find(&self, code: &str) -> Option<MeasurementEstimate> {
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

    fn save(&self, item: &MeasurementEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_estimate_store(
    inner: Rc<dyn MeasurementEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementEstimateStore> {
    Rc::new(CachedMeasurementEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementEstimate>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEstimateQuery {
    pub category: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementEstimateQuery {
    pub fn matches(&self, item: &MeasurementEstimate) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementEstimate]) -> Vec<MeasurementEstimate> {
        let matched = collections::filter(items, |candidate: &MeasurementEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_estimates(
    store: Rc<dyn MeasurementEstimateStore>,
    query: &MeasurementEstimateQuery,
) -> Vec<MeasurementEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MeasurementEstimateStore> = new_memory_measurement_estimate_store();
        let item = new_measurement_estimate("code-921", "name-467");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
