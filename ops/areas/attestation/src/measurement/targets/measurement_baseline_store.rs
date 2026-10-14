use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement baselines by code.
pub trait MeasurementBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementBaseline>;
    fn save(&self, item: &MeasurementBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementBaselineStore {
    pub items: RefCell<BTreeMap<String, MeasurementBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementBaselineStore for MemoryMeasurementBaselineStore {
    fn find(&self, code: &str) -> Option<MeasurementBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementBaseline) {
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

    fn list(&self) -> Vec<MeasurementBaseline> {
        let mut out: Vec<MeasurementBaseline> = Vec::new();
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

pub fn new_memory_measurement_baseline_store() -> Rc<dyn MeasurementBaselineStore> {
    Rc::new(MemoryMeasurementBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementBaselineStore {
    pub inner: Rc<dyn MeasurementBaselineStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementBaselineStore for CachedMeasurementBaselineStore {
    fn find(&self, code: &str) -> Option<MeasurementBaseline> {
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

    fn save(&self, item: &MeasurementBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_baseline_store(
    inner: Rc<dyn MeasurementBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementBaselineStore> {
    Rc::new(CachedMeasurementBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementBaseline>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBaselineQuery {
    pub unit: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementBaselineQuery {
    pub fn matches(&self, item: &MeasurementBaseline) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementBaseline]) -> Vec<MeasurementBaseline> {
        let matched = collections::filter(items, |candidate: &MeasurementBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_baselines(
    store: Rc<dyn MeasurementBaselineStore>,
    query: &MeasurementBaselineQuery,
) -> Vec<MeasurementBaseline> {
    query.apply(&store.list())
}
