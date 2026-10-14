use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy measurement baselines by code.
pub trait LegacyMeasurementBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyMeasurementBaseline>;
    fn save(&self, item: &LegacyMeasurementBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyMeasurementBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyMeasurementBaselineStore {
    pub items: RefCell<BTreeMap<String, LegacyMeasurementBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyMeasurementBaselineStore for MemoryLegacyMeasurementBaselineStore {
    fn find(&self, code: &str) -> Option<LegacyMeasurementBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyMeasurementBaseline) {
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

    fn list(&self) -> Vec<LegacyMeasurementBaseline> {
        let mut out: Vec<LegacyMeasurementBaseline> = Vec::new();
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

pub fn new_memory_legacy_measurement_baseline_store() -> Rc<dyn LegacyMeasurementBaselineStore> {
    Rc::new(MemoryLegacyMeasurementBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyMeasurementBaselineStore {
    pub inner: Rc<dyn LegacyMeasurementBaselineStore>,
    pub cache: RefCell<cache::TtlCache<LegacyMeasurementBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyMeasurementBaselineStore for CachedLegacyMeasurementBaselineStore {
    fn find(&self, code: &str) -> Option<LegacyMeasurementBaseline> {
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

    fn save(&self, item: &LegacyMeasurementBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyMeasurementBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_measurement_baseline_store(
    inner: Rc<dyn LegacyMeasurementBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyMeasurementBaselineStore> {
    Rc::new(CachedLegacyMeasurementBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyMeasurementBaseline>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy measurement baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBaselineQuery {
    pub label: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyMeasurementBaselineQuery {
    pub fn matches(&self, item: &LegacyMeasurementBaseline) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyMeasurementBaseline]) -> Vec<LegacyMeasurementBaseline> {
        let matched = collections::filter(items, |candidate: &LegacyMeasurementBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_measurement_baselines(
    store: Rc<dyn LegacyMeasurementBaselineStore>,
    query: &LegacyMeasurementBaselineQuery,
) -> Vec<LegacyMeasurementBaseline> {
    query.apply(&store.list())
}
