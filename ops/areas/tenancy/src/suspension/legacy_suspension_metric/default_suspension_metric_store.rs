use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default suspension metrics by code.
pub trait DefaultSuspensionMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultSuspensionMetric>;
    fn save(&self, item: &DefaultSuspensionMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultSuspensionMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultSuspensionMetricStore {
    pub items: RefCell<BTreeMap<String, DefaultSuspensionMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultSuspensionMetricStore for MemoryDefaultSuspensionMetricStore {
    fn find(&self, code: &str) -> Option<DefaultSuspensionMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultSuspensionMetric) {
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

    fn list(&self) -> Vec<DefaultSuspensionMetric> {
        let mut out: Vec<DefaultSuspensionMetric> = Vec::new();
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

pub fn new_memory_default_suspension_metric_store() -> Rc<dyn DefaultSuspensionMetricStore> {
    Rc::new(MemoryDefaultSuspensionMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultSuspensionMetricStore {
    pub inner: Rc<dyn DefaultSuspensionMetricStore>,
    pub cache: RefCell<cache::TtlCache<DefaultSuspensionMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultSuspensionMetricStore for CachedDefaultSuspensionMetricStore {
    fn find(&self, code: &str) -> Option<DefaultSuspensionMetric> {
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

    fn save(&self, item: &DefaultSuspensionMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultSuspensionMetric> {
        self.inner.list()
    }
}

pub fn new_cached_default_suspension_metric_store(
    inner: Rc<dyn DefaultSuspensionMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultSuspensionMetricStore> {
    Rc::new(CachedDefaultSuspensionMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultSuspensionMetric>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default suspension metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionMetricQuery {
    pub title: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultSuspensionMetricQuery {
    pub fn matches(&self, item: &DefaultSuspensionMetric) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[DefaultSuspensionMetric]) -> Vec<DefaultSuspensionMetric> {
        let matched = collections::filter(items, |candidate: &DefaultSuspensionMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_suspension_metrics(
    store: Rc<dyn DefaultSuspensionMetricStore>,
    query: &DefaultSuspensionMetricQuery,
) -> Vec<DefaultSuspensionMetric> {
    query.apply(&store.list())
}
