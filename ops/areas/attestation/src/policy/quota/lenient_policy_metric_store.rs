use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient policy metrics by code.
pub trait LenientPolicyMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientPolicyMetric>;
    fn save(&self, item: &LenientPolicyMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientPolicyMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientPolicyMetricStore {
    pub items: RefCell<BTreeMap<String, LenientPolicyMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientPolicyMetricStore for MemoryLenientPolicyMetricStore {
    fn find(&self, code: &str) -> Option<LenientPolicyMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientPolicyMetric) {
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

    fn list(&self) -> Vec<LenientPolicyMetric> {
        let mut out: Vec<LenientPolicyMetric> = Vec::new();
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

pub fn new_memory_lenient_policy_metric_store() -> Rc<dyn LenientPolicyMetricStore> {
    Rc::new(MemoryLenientPolicyMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientPolicyMetricStore {
    pub inner: Rc<dyn LenientPolicyMetricStore>,
    pub cache: RefCell<cache::TtlCache<LenientPolicyMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientPolicyMetricStore for CachedLenientPolicyMetricStore {
    fn find(&self, code: &str) -> Option<LenientPolicyMetric> {
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

    fn save(&self, item: &LenientPolicyMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientPolicyMetric> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_policy_metric_store(
    inner: Rc<dyn LenientPolicyMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientPolicyMetricStore> {
    Rc::new(CachedLenientPolicyMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientPolicyMetric>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient policy metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyMetricQuery {
    pub channel: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientPolicyMetricQuery {
    pub fn matches(&self, item: &LenientPolicyMetric) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientPolicyMetric]) -> Vec<LenientPolicyMetric> {
        let matched = collections::filter(items, |candidate: &LenientPolicyMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_policy_metrics(
    store: Rc<dyn LenientPolicyMetricStore>,
    query: &LenientPolicyMetricQuery,
) -> Vec<LenientPolicyMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientPolicyMetricStore> = new_memory_lenient_policy_metric_store();
        let item = new_lenient_policy_metric("code-15", "title-402");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
