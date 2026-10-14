use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped segment metrics by code.
pub trait CappedSegmentMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedSegmentMetric>;
    fn save(&self, item: &CappedSegmentMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedSegmentMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedSegmentMetricStore {
    pub items: RefCell<BTreeMap<String, CappedSegmentMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedSegmentMetricStore for MemoryCappedSegmentMetricStore {
    fn find(&self, code: &str) -> Option<CappedSegmentMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedSegmentMetric) {
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

    fn list(&self) -> Vec<CappedSegmentMetric> {
        let mut out: Vec<CappedSegmentMetric> = Vec::new();
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

pub fn new_memory_capped_segment_metric_store() -> Rc<dyn CappedSegmentMetricStore> {
    Rc::new(MemoryCappedSegmentMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedSegmentMetricStore {
    pub inner: Rc<dyn CappedSegmentMetricStore>,
    pub cache: RefCell<cache::TtlCache<CappedSegmentMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedSegmentMetricStore for CachedCappedSegmentMetricStore {
    fn find(&self, code: &str) -> Option<CappedSegmentMetric> {
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

    fn save(&self, item: &CappedSegmentMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedSegmentMetric> {
        self.inner.list()
    }
}

pub fn new_cached_capped_segment_metric_store(
    inner: Rc<dyn CappedSegmentMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedSegmentMetricStore> {
    Rc::new(CachedCappedSegmentMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedSegmentMetric>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped segment metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentMetricQuery {
    pub locale: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedSegmentMetricQuery {
    pub fn matches(&self, item: &CappedSegmentMetric) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedSegmentMetric]) -> Vec<CappedSegmentMetric> {
        let matched = collections::filter(items, |candidate: &CappedSegmentMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_segment_metrics(
    store: Rc<dyn CappedSegmentMetricStore>,
    query: &CappedSegmentMetricQuery,
) -> Vec<CappedSegmentMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedSegmentMetricStore> = new_memory_capped_segment_metric_store();
        let item = new_capped_segment_metric("code-479", "label-53");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
