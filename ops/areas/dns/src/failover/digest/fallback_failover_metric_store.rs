use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback failover metrics by code.
pub trait FallbackFailoverMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackFailoverMetric>;
    fn save(&self, item: &FallbackFailoverMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackFailoverMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackFailoverMetricStore {
    pub items: RefCell<BTreeMap<String, FallbackFailoverMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackFailoverMetricStore for MemoryFallbackFailoverMetricStore {
    fn find(&self, code: &str) -> Option<FallbackFailoverMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackFailoverMetric) {
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

    fn list(&self) -> Vec<FallbackFailoverMetric> {
        let mut out: Vec<FallbackFailoverMetric> = Vec::new();
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

pub fn new_memory_fallback_failover_metric_store() -> Rc<dyn FallbackFailoverMetricStore> {
    Rc::new(MemoryFallbackFailoverMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackFailoverMetricStore {
    pub inner: Rc<dyn FallbackFailoverMetricStore>,
    pub cache: RefCell<cache::TtlCache<FallbackFailoverMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackFailoverMetricStore for CachedFallbackFailoverMetricStore {
    fn find(&self, code: &str) -> Option<FallbackFailoverMetric> {
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

    fn save(&self, item: &FallbackFailoverMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackFailoverMetric> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_failover_metric_store(
    inner: Rc<dyn FallbackFailoverMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackFailoverMetricStore> {
    Rc::new(CachedFallbackFailoverMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackFailoverMetric>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback failover metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverMetricQuery {
    pub segment: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackFailoverMetricQuery {
    pub fn matches(&self, item: &FallbackFailoverMetric) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackFailoverMetric]) -> Vec<FallbackFailoverMetric> {
        let matched = collections::filter(items, |candidate: &FallbackFailoverMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_failover_metrics(
    store: Rc<dyn FallbackFailoverMetricStore>,
    query: &FallbackFailoverMetricQuery,
) -> Vec<FallbackFailoverMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackFailoverMetricStore> = new_memory_fallback_failover_metric_store();
        let item = new_fallback_failover_metric("code-708", "name-738");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
