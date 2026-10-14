use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted alert metrics by code.
pub trait WeightedAlertMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedAlertMetric>;
    fn save(&self, item: &WeightedAlertMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedAlertMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedAlertMetricStore {
    pub items: RefCell<BTreeMap<String, WeightedAlertMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedAlertMetricStore for MemoryWeightedAlertMetricStore {
    fn find(&self, code: &str) -> Option<WeightedAlertMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedAlertMetric) {
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

    fn list(&self) -> Vec<WeightedAlertMetric> {
        let mut out: Vec<WeightedAlertMetric> = Vec::new();
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

pub fn new_memory_weighted_alert_metric_store() -> Rc<dyn WeightedAlertMetricStore> {
    Rc::new(MemoryWeightedAlertMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedAlertMetricStore {
    pub inner: Rc<dyn WeightedAlertMetricStore>,
    pub cache: RefCell<cache::TtlCache<WeightedAlertMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedAlertMetricStore for CachedWeightedAlertMetricStore {
    fn find(&self, code: &str) -> Option<WeightedAlertMetric> {
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

    fn save(&self, item: &WeightedAlertMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedAlertMetric> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_alert_metric_store(
    inner: Rc<dyn WeightedAlertMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedAlertMetricStore> {
    Rc::new(CachedWeightedAlertMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedAlertMetric>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted alert metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertMetricQuery {
    pub label: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedAlertMetricQuery {
    pub fn matches(&self, item: &WeightedAlertMetric) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedAlertMetric]) -> Vec<WeightedAlertMetric> {
        let matched = collections::filter(items, |candidate: &WeightedAlertMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_alert_metrics(
    store: Rc<dyn WeightedAlertMetricStore>,
    query: &WeightedAlertMetricQuery,
) -> Vec<WeightedAlertMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedAlertMetricStore> = new_memory_weighted_alert_metric_store();
        let item = new_weighted_alert_metric("code-97", "label-350");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
