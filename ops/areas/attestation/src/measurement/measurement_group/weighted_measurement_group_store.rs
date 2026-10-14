use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted measurement groups by code.
pub trait WeightedMeasurementGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedMeasurementGroup>;
    fn save(&self, item: &WeightedMeasurementGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedMeasurementGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedMeasurementGroupStore {
    pub items: RefCell<BTreeMap<String, WeightedMeasurementGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedMeasurementGroupStore for MemoryWeightedMeasurementGroupStore {
    fn find(&self, code: &str) -> Option<WeightedMeasurementGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedMeasurementGroup) {
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

    fn list(&self) -> Vec<WeightedMeasurementGroup> {
        let mut out: Vec<WeightedMeasurementGroup> = Vec::new();
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

pub fn new_memory_weighted_measurement_group_store() -> Rc<dyn WeightedMeasurementGroupStore> {
    Rc::new(MemoryWeightedMeasurementGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedMeasurementGroupStore {
    pub inner: Rc<dyn WeightedMeasurementGroupStore>,
    pub cache: RefCell<cache::TtlCache<WeightedMeasurementGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedMeasurementGroupStore for CachedWeightedMeasurementGroupStore {
    fn find(&self, code: &str) -> Option<WeightedMeasurementGroup> {
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

    fn save(&self, item: &WeightedMeasurementGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedMeasurementGroup> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_measurement_group_store(
    inner: Rc<dyn WeightedMeasurementGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedMeasurementGroupStore> {
    Rc::new(CachedWeightedMeasurementGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedMeasurementGroup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted measurement groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementGroupQuery {
    pub name: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedMeasurementGroupQuery {
    pub fn matches(&self, item: &WeightedMeasurementGroup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedMeasurementGroup]) -> Vec<WeightedMeasurementGroup> {
        let matched = collections::filter(items, |candidate: &WeightedMeasurementGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_measurement_groups(
    store: Rc<dyn WeightedMeasurementGroupStore>,
    query: &WeightedMeasurementGroupQuery,
) -> Vec<WeightedMeasurementGroup> {
    query.apply(&store.list())
}
