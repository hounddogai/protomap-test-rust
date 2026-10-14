use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient measurement rollups by code.
pub trait LenientMeasurementRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientMeasurementRollup>;
    fn save(&self, item: &LenientMeasurementRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientMeasurementRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientMeasurementRollupStore {
    pub items: RefCell<BTreeMap<String, LenientMeasurementRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientMeasurementRollupStore for MemoryLenientMeasurementRollupStore {
    fn find(&self, code: &str) -> Option<LenientMeasurementRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientMeasurementRollup) {
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

    fn list(&self) -> Vec<LenientMeasurementRollup> {
        let mut out: Vec<LenientMeasurementRollup> = Vec::new();
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

pub fn new_memory_lenient_measurement_rollup_store() -> Rc<dyn LenientMeasurementRollupStore> {
    Rc::new(MemoryLenientMeasurementRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientMeasurementRollupStore {
    pub inner: Rc<dyn LenientMeasurementRollupStore>,
    pub cache: RefCell<cache::TtlCache<LenientMeasurementRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientMeasurementRollupStore for CachedLenientMeasurementRollupStore {
    fn find(&self, code: &str) -> Option<LenientMeasurementRollup> {
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

    fn save(&self, item: &LenientMeasurementRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientMeasurementRollup> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_measurement_rollup_store(
    inner: Rc<dyn LenientMeasurementRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientMeasurementRollupStore> {
    Rc::new(CachedLenientMeasurementRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientMeasurementRollup>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient measurement rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementRollupQuery {
    pub label: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientMeasurementRollupQuery {
    pub fn matches(&self, item: &LenientMeasurementRollup) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientMeasurementRollup]) -> Vec<LenientMeasurementRollup> {
        let matched = collections::filter(items, |candidate: &LenientMeasurementRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_measurement_rollups(
    store: Rc<dyn LenientMeasurementRollupStore>,
    query: &LenientMeasurementRollupQuery,
) -> Vec<LenientMeasurementRollup> {
    query.apply(&store.list())
}
