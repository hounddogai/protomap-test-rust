use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan metrics by code.
pub trait PlanMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanMetric>;
    fn save(&self, item: &PlanMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanMetricStore {
    pub items: RefCell<BTreeMap<String, PlanMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanMetricStore for MemoryPlanMetricStore {
    fn find(&self, code: &str) -> Option<PlanMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanMetric) {
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

    fn list(&self) -> Vec<PlanMetric> {
        let mut out: Vec<PlanMetric> = Vec::new();
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

pub fn new_memory_plan_metric_store() -> Rc<dyn PlanMetricStore> {
    Rc::new(MemoryPlanMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanMetricStore {
    pub inner: Rc<dyn PlanMetricStore>,
    pub cache: RefCell<cache::TtlCache<PlanMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanMetricStore for CachedPlanMetricStore {
    fn find(&self, code: &str) -> Option<PlanMetric> {
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

    fn save(&self, item: &PlanMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanMetric> {
        self.inner.list()
    }
}

pub fn new_cached_plan_metric_store(
    inner: Rc<dyn PlanMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanMetricStore> {
    Rc::new(CachedPlanMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanMetric>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanMetricQuery {
    pub title: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanMetricQuery {
    pub fn matches(&self, item: &PlanMetric) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PlanMetric]) -> Vec<PlanMetric> {
        let matched = collections::filter(items, |candidate: &PlanMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_metrics(store: Rc<dyn PlanMetricStore>, query: &PlanMetricQuery) -> Vec<PlanMetric> {
    query.apply(&store.list())
}
