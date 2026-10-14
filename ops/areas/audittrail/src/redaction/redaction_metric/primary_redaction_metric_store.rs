use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary redaction metrics by code.
pub trait PrimaryRedactionMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryRedactionMetric>;
    fn save(&self, item: &PrimaryRedactionMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryRedactionMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryRedactionMetricStore {
    pub items: RefCell<BTreeMap<String, PrimaryRedactionMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryRedactionMetricStore for MemoryPrimaryRedactionMetricStore {
    fn find(&self, code: &str) -> Option<PrimaryRedactionMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryRedactionMetric) {
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

    fn list(&self) -> Vec<PrimaryRedactionMetric> {
        let mut out: Vec<PrimaryRedactionMetric> = Vec::new();
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

pub fn new_memory_primary_redaction_metric_store() -> Rc<dyn PrimaryRedactionMetricStore> {
    Rc::new(MemoryPrimaryRedactionMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryRedactionMetricStore {
    pub inner: Rc<dyn PrimaryRedactionMetricStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryRedactionMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryRedactionMetricStore for CachedPrimaryRedactionMetricStore {
    fn find(&self, code: &str) -> Option<PrimaryRedactionMetric> {
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

    fn save(&self, item: &PrimaryRedactionMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryRedactionMetric> {
        self.inner.list()
    }
}

pub fn new_cached_primary_redaction_metric_store(
    inner: Rc<dyn PrimaryRedactionMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryRedactionMetricStore> {
    Rc::new(CachedPrimaryRedactionMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryRedactionMetric>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary redaction metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionMetricQuery {
    pub cluster: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryRedactionMetricQuery {
    pub fn matches(&self, item: &PrimaryRedactionMetric) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryRedactionMetric]) -> Vec<PrimaryRedactionMetric> {
        let matched = collections::filter(items, |candidate: &PrimaryRedactionMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_redaction_metrics(
    store: Rc<dyn PrimaryRedactionMetricStore>,
    query: &PrimaryRedactionMetricQuery,
) -> Vec<PrimaryRedactionMetric> {
    query.apply(&store.list())
}
