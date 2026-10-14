use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary retention metrics by code.
pub trait PrimaryRetentionMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryRetentionMetric>;
    fn save(&self, item: &PrimaryRetentionMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryRetentionMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryRetentionMetricStore {
    pub items: RefCell<BTreeMap<String, PrimaryRetentionMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryRetentionMetricStore for MemoryPrimaryRetentionMetricStore {
    fn find(&self, code: &str) -> Option<PrimaryRetentionMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryRetentionMetric) {
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

    fn list(&self) -> Vec<PrimaryRetentionMetric> {
        let mut out: Vec<PrimaryRetentionMetric> = Vec::new();
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

pub fn new_memory_primary_retention_metric_store() -> Rc<dyn PrimaryRetentionMetricStore> {
    Rc::new(MemoryPrimaryRetentionMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryRetentionMetricStore {
    pub inner: Rc<dyn PrimaryRetentionMetricStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryRetentionMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryRetentionMetricStore for CachedPrimaryRetentionMetricStore {
    fn find(&self, code: &str) -> Option<PrimaryRetentionMetric> {
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

    fn save(&self, item: &PrimaryRetentionMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryRetentionMetric> {
        self.inner.list()
    }
}

pub fn new_cached_primary_retention_metric_store(
    inner: Rc<dyn PrimaryRetentionMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryRetentionMetricStore> {
    Rc::new(CachedPrimaryRetentionMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryRetentionMetric>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary retention metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionMetricQuery {
    pub country: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryRetentionMetricQuery {
    pub fn matches(&self, item: &PrimaryRetentionMetric) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryRetentionMetric]) -> Vec<PrimaryRetentionMetric> {
        let matched = collections::filter(items, |candidate: &PrimaryRetentionMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_retention_metrics(
    store: Rc<dyn PrimaryRetentionMetricStore>,
    query: &PrimaryRetentionMetricQuery,
) -> Vec<PrimaryRetentionMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryRetentionMetricStore> = new_memory_primary_retention_metric_store();
        let item = new_primary_retention_metric("code-345", "name-253");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
