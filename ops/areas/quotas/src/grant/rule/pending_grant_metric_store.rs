use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending grant metrics by code.
pub trait PendingGrantMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingGrantMetric>;
    fn save(&self, item: &PendingGrantMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingGrantMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingGrantMetricStore {
    pub items: RefCell<BTreeMap<String, PendingGrantMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingGrantMetricStore for MemoryPendingGrantMetricStore {
    fn find(&self, code: &str) -> Option<PendingGrantMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingGrantMetric) {
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

    fn list(&self) -> Vec<PendingGrantMetric> {
        let mut out: Vec<PendingGrantMetric> = Vec::new();
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

pub fn new_memory_pending_grant_metric_store() -> Rc<dyn PendingGrantMetricStore> {
    Rc::new(MemoryPendingGrantMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingGrantMetricStore {
    pub inner: Rc<dyn PendingGrantMetricStore>,
    pub cache: RefCell<cache::TtlCache<PendingGrantMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingGrantMetricStore for CachedPendingGrantMetricStore {
    fn find(&self, code: &str) -> Option<PendingGrantMetric> {
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

    fn save(&self, item: &PendingGrantMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingGrantMetric> {
        self.inner.list()
    }
}

pub fn new_cached_pending_grant_metric_store(
    inner: Rc<dyn PendingGrantMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingGrantMetricStore> {
    Rc::new(CachedPendingGrantMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingGrantMetric>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending grant metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantMetricQuery {
    pub country: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingGrantMetricQuery {
    pub fn matches(&self, item: &PendingGrantMetric) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[PendingGrantMetric]) -> Vec<PendingGrantMetric> {
        let matched = collections::filter(items, |candidate: &PendingGrantMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_grant_metrics(
    store: Rc<dyn PendingGrantMetricStore>,
    query: &PendingGrantMetricQuery,
) -> Vec<PendingGrantMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingGrantMetricStore> = new_memory_pending_grant_metric_store();
        let item = new_pending_grant_metric("code-594", "title-324");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
