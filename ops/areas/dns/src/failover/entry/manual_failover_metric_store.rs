use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual failover metrics by code.
pub trait ManualFailoverMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualFailoverMetric>;
    fn save(&self, item: &ManualFailoverMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualFailoverMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualFailoverMetricStore {
    pub items: RefCell<BTreeMap<String, ManualFailoverMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualFailoverMetricStore for MemoryManualFailoverMetricStore {
    fn find(&self, code: &str) -> Option<ManualFailoverMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualFailoverMetric) {
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

    fn list(&self) -> Vec<ManualFailoverMetric> {
        let mut out: Vec<ManualFailoverMetric> = Vec::new();
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

pub fn new_memory_manual_failover_metric_store() -> Rc<dyn ManualFailoverMetricStore> {
    Rc::new(MemoryManualFailoverMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualFailoverMetricStore {
    pub inner: Rc<dyn ManualFailoverMetricStore>,
    pub cache: RefCell<cache::TtlCache<ManualFailoverMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualFailoverMetricStore for CachedManualFailoverMetricStore {
    fn find(&self, code: &str) -> Option<ManualFailoverMetric> {
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

    fn save(&self, item: &ManualFailoverMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualFailoverMetric> {
        self.inner.list()
    }
}

pub fn new_cached_manual_failover_metric_store(
    inner: Rc<dyn ManualFailoverMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualFailoverMetricStore> {
    Rc::new(CachedManualFailoverMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualFailoverMetric>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual failover metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverMetricQuery {
    pub source: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualFailoverMetricQuery {
    pub fn matches(&self, item: &ManualFailoverMetric) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualFailoverMetric]) -> Vec<ManualFailoverMetric> {
        let matched = collections::filter(items, |candidate: &ManualFailoverMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_failover_metrics(
    store: Rc<dyn ManualFailoverMetricStore>,
    query: &ManualFailoverMetricQuery,
) -> Vec<ManualFailoverMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualFailoverMetricStore> = new_memory_manual_failover_metric_store();
        let item = new_manual_failover_metric("code-42", "title-57");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
