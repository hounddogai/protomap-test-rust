use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive data export metrics by code.
pub trait AdaptiveDataExportMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveDataExportMetric>;
    fn save(&self, item: &AdaptiveDataExportMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveDataExportMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveDataExportMetricStore {
    pub items: RefCell<BTreeMap<String, AdaptiveDataExportMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveDataExportMetricStore for MemoryAdaptiveDataExportMetricStore {
    fn find(&self, code: &str) -> Option<AdaptiveDataExportMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveDataExportMetric) {
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

    fn list(&self) -> Vec<AdaptiveDataExportMetric> {
        let mut out: Vec<AdaptiveDataExportMetric> = Vec::new();
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

pub fn new_memory_adaptive_data_export_metric_store() -> Rc<dyn AdaptiveDataExportMetricStore> {
    Rc::new(MemoryAdaptiveDataExportMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveDataExportMetricStore {
    pub inner: Rc<dyn AdaptiveDataExportMetricStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveDataExportMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveDataExportMetricStore for CachedAdaptiveDataExportMetricStore {
    fn find(&self, code: &str) -> Option<AdaptiveDataExportMetric> {
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

    fn save(&self, item: &AdaptiveDataExportMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveDataExportMetric> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_data_export_metric_store(
    inner: Rc<dyn AdaptiveDataExportMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveDataExportMetricStore> {
    Rc::new(CachedAdaptiveDataExportMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveDataExportMetric>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive data export metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDataExportMetricQuery {
    pub label: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveDataExportMetricQuery {
    pub fn matches(&self, item: &AdaptiveDataExportMetric) -> bool {
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

    pub fn apply(&self, items: &[AdaptiveDataExportMetric]) -> Vec<AdaptiveDataExportMetric> {
        let matched = collections::filter(items, |candidate: &AdaptiveDataExportMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_data_export_metrics(
    store: Rc<dyn AdaptiveDataExportMetricStore>,
    query: &AdaptiveDataExportMetricQuery,
) -> Vec<AdaptiveDataExportMetric> {
    query.apply(&store.list())
}
