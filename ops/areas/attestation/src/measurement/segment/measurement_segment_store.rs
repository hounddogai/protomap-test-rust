use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement segments by code.
pub trait MeasurementSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementSegment>;
    fn save(&self, item: &MeasurementSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementSegmentStore {
    pub items: RefCell<BTreeMap<String, MeasurementSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementSegmentStore for MemoryMeasurementSegmentStore {
    fn find(&self, code: &str) -> Option<MeasurementSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementSegment) {
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

    fn list(&self) -> Vec<MeasurementSegment> {
        let mut out: Vec<MeasurementSegment> = Vec::new();
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

pub fn new_memory_measurement_segment_store() -> Rc<dyn MeasurementSegmentStore> {
    Rc::new(MemoryMeasurementSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementSegmentStore {
    pub inner: Rc<dyn MeasurementSegmentStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementSegmentStore for CachedMeasurementSegmentStore {
    fn find(&self, code: &str) -> Option<MeasurementSegment> {
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

    fn save(&self, item: &MeasurementSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementSegment> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_segment_store(
    inner: Rc<dyn MeasurementSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementSegmentStore> {
    Rc::new(CachedMeasurementSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementSegment>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSegmentQuery {
    pub category: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementSegmentQuery {
    pub fn matches(&self, item: &MeasurementSegment) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[MeasurementSegment]) -> Vec<MeasurementSegment> {
        let matched = collections::filter(items, |candidate: &MeasurementSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_segments(
    store: Rc<dyn MeasurementSegmentStore>,
    query: &MeasurementSegmentQuery,
) -> Vec<MeasurementSegment> {
    query.apply(&store.list())
}
