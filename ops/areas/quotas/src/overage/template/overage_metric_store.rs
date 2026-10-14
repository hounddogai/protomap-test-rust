use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps overage metrics by code.
pub trait OverageMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<OverageMetric>;
    fn save(&self, item: &OverageMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<OverageMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryOverageMetricStore {
    pub items: RefCell<BTreeMap<String, OverageMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl OverageMetricStore for MemoryOverageMetricStore {
    fn find(&self, code: &str) -> Option<OverageMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &OverageMetric) {
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

    fn list(&self) -> Vec<OverageMetric> {
        let mut out: Vec<OverageMetric> = Vec::new();
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

pub fn new_memory_overage_metric_store() -> Rc<dyn OverageMetricStore> {
    Rc::new(MemoryOverageMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedOverageMetricStore {
    pub inner: Rc<dyn OverageMetricStore>,
    pub cache: RefCell<cache::TtlCache<OverageMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl OverageMetricStore for CachedOverageMetricStore {
    fn find(&self, code: &str) -> Option<OverageMetric> {
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

    fn save(&self, item: &OverageMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<OverageMetric> {
        self.inner.list()
    }
}

pub fn new_cached_overage_metric_store(
    inner: Rc<dyn OverageMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn OverageMetricStore> {
    Rc::new(CachedOverageMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<OverageMetric>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects overage metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageMetricQuery {
    pub cluster: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl OverageMetricQuery {
    pub fn matches(&self, item: &OverageMetric) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[OverageMetric]) -> Vec<OverageMetric> {
        let matched = collections::filter(items, |candidate: &OverageMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_overage_metrics(store: Rc<dyn OverageMetricStore>, query: &OverageMetricQuery) -> Vec<OverageMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn OverageMetricStore> = new_memory_overage_metric_store();
        let item = new_overage_metric("code-650", "label-556");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
