use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps egress metrics by code.
pub trait EgressMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EgressMetric>;
    fn save(&self, item: &EgressMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EgressMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEgressMetricStore {
    pub items: RefCell<BTreeMap<String, EgressMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl EgressMetricStore for MemoryEgressMetricStore {
    fn find(&self, code: &str) -> Option<EgressMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EgressMetric) {
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

    fn list(&self) -> Vec<EgressMetric> {
        let mut out: Vec<EgressMetric> = Vec::new();
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

pub fn new_memory_egress_metric_store() -> Rc<dyn EgressMetricStore> {
    Rc::new(MemoryEgressMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEgressMetricStore {
    pub inner: Rc<dyn EgressMetricStore>,
    pub cache: RefCell<cache::TtlCache<EgressMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EgressMetricStore for CachedEgressMetricStore {
    fn find(&self, code: &str) -> Option<EgressMetric> {
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

    fn save(&self, item: &EgressMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EgressMetric> {
        self.inner.list()
    }
}

pub fn new_cached_egress_metric_store(
    inner: Rc<dyn EgressMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EgressMetricStore> {
    Rc::new(CachedEgressMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EgressMetric>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects egress metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressMetricQuery {
    pub locale: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EgressMetricQuery {
    pub fn matches(&self, item: &EgressMetric) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EgressMetric]) -> Vec<EgressMetric> {
        let matched = collections::filter(items, |candidate: &EgressMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_egress_metrics(store: Rc<dyn EgressMetricStore>, query: &EgressMetricQuery) -> Vec<EgressMetric> {
    query.apply(&store.list())
}
