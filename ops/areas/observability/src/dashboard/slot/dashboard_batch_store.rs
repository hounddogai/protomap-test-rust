use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dashboard batches by code.
pub trait DashboardBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DashboardBatch>;
    fn save(&self, item: &DashboardBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DashboardBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDashboardBatchStore {
    pub items: RefCell<BTreeMap<String, DashboardBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl DashboardBatchStore for MemoryDashboardBatchStore {
    fn find(&self, code: &str) -> Option<DashboardBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DashboardBatch) {
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

    fn list(&self) -> Vec<DashboardBatch> {
        let mut out: Vec<DashboardBatch> = Vec::new();
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

pub fn new_memory_dashboard_batch_store() -> Rc<dyn DashboardBatchStore> {
    Rc::new(MemoryDashboardBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDashboardBatchStore {
    pub inner: Rc<dyn DashboardBatchStore>,
    pub cache: RefCell<cache::TtlCache<DashboardBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DashboardBatchStore for CachedDashboardBatchStore {
    fn find(&self, code: &str) -> Option<DashboardBatch> {
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

    fn save(&self, item: &DashboardBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DashboardBatch> {
        self.inner.list()
    }
}

pub fn new_cached_dashboard_batch_store(
    inner: Rc<dyn DashboardBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DashboardBatchStore> {
    Rc::new(CachedDashboardBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DashboardBatch>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dashboard batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBatchQuery {
    pub unit: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DashboardBatchQuery {
    pub fn matches(&self, item: &DashboardBatch) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DashboardBatch]) -> Vec<DashboardBatch> {
        let matched = collections::filter(items, |candidate: &DashboardBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dashboard_batches(store: Rc<dyn DashboardBatchStore>, query: &DashboardBatchQuery) -> Vec<DashboardBatch> {
    query.apply(&store.list())
}
