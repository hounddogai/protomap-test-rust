use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast segments by code.
pub trait ForecastSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastSegment>;
    fn save(&self, item: &ForecastSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastSegmentStore {
    pub items: RefCell<BTreeMap<String, ForecastSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastSegmentStore for MemoryForecastSegmentStore {
    fn find(&self, code: &str) -> Option<ForecastSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastSegment) {
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

    fn list(&self) -> Vec<ForecastSegment> {
        let mut out: Vec<ForecastSegment> = Vec::new();
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

pub fn new_memory_forecast_segment_store() -> Rc<dyn ForecastSegmentStore> {
    Rc::new(MemoryForecastSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastSegmentStore {
    pub inner: Rc<dyn ForecastSegmentStore>,
    pub cache: RefCell<cache::TtlCache<ForecastSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastSegmentStore for CachedForecastSegmentStore {
    fn find(&self, code: &str) -> Option<ForecastSegment> {
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

    fn save(&self, item: &ForecastSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastSegment> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_segment_store(
    inner: Rc<dyn ForecastSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastSegmentStore> {
    Rc::new(CachedForecastSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastSegment>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSegmentQuery {
    pub title: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastSegmentQuery {
    pub fn matches(&self, item: &ForecastSegment) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[ForecastSegment]) -> Vec<ForecastSegment> {
        let matched = collections::filter(items, |candidate: &ForecastSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_segments(
    store: Rc<dyn ForecastSegmentStore>,
    query: &ForecastSegmentQuery,
) -> Vec<ForecastSegment> {
    query.apply(&store.list())
}
