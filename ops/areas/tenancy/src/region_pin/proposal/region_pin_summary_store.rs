use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps region pin summaries by code.
pub trait RegionPinSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionPinSummary>;
    fn save(&self, item: &RegionPinSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionPinSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionPinSummaryStore {
    pub items: RefCell<BTreeMap<String, RegionPinSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionPinSummaryStore for MemoryRegionPinSummaryStore {
    fn find(&self, code: &str) -> Option<RegionPinSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionPinSummary) {
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

    fn list(&self) -> Vec<RegionPinSummary> {
        let mut out: Vec<RegionPinSummary> = Vec::new();
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

pub fn new_memory_region_pin_summary_store() -> Rc<dyn RegionPinSummaryStore> {
    Rc::new(MemoryRegionPinSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionPinSummaryStore {
    pub inner: Rc<dyn RegionPinSummaryStore>,
    pub cache: RefCell<cache::TtlCache<RegionPinSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionPinSummaryStore for CachedRegionPinSummaryStore {
    fn find(&self, code: &str) -> Option<RegionPinSummary> {
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

    fn save(&self, item: &RegionPinSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionPinSummary> {
        self.inner.list()
    }
}

pub fn new_cached_region_pin_summary_store(
    inner: Rc<dyn RegionPinSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionPinSummaryStore> {
    Rc::new(CachedRegionPinSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionPinSummary>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects region pin summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSummaryQuery {
    pub channel: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionPinSummaryQuery {
    pub fn matches(&self, item: &RegionPinSummary) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionPinSummary]) -> Vec<RegionPinSummary> {
        let matched = collections::filter(items, |candidate: &RegionPinSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_region_pin_summaries(
    store: Rc<dyn RegionPinSummaryStore>,
    query: &RegionPinSummaryQuery,
) -> Vec<RegionPinSummary> {
    query.apply(&store.list())
}
