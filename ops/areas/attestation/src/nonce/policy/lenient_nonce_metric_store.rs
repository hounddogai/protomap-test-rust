use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient nonce metrics by code.
pub trait LenientNonceMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientNonceMetric>;
    fn save(&self, item: &LenientNonceMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientNonceMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientNonceMetricStore {
    pub items: RefCell<BTreeMap<String, LenientNonceMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientNonceMetricStore for MemoryLenientNonceMetricStore {
    fn find(&self, code: &str) -> Option<LenientNonceMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientNonceMetric) {
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

    fn list(&self) -> Vec<LenientNonceMetric> {
        let mut out: Vec<LenientNonceMetric> = Vec::new();
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

pub fn new_memory_lenient_nonce_metric_store() -> Rc<dyn LenientNonceMetricStore> {
    Rc::new(MemoryLenientNonceMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientNonceMetricStore {
    pub inner: Rc<dyn LenientNonceMetricStore>,
    pub cache: RefCell<cache::TtlCache<LenientNonceMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientNonceMetricStore for CachedLenientNonceMetricStore {
    fn find(&self, code: &str) -> Option<LenientNonceMetric> {
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

    fn save(&self, item: &LenientNonceMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientNonceMetric> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_nonce_metric_store(
    inner: Rc<dyn LenientNonceMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientNonceMetricStore> {
    Rc::new(CachedLenientNonceMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientNonceMetric>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient nonce metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceMetricQuery {
    pub category: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientNonceMetricQuery {
    pub fn matches(&self, item: &LenientNonceMetric) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientNonceMetric]) -> Vec<LenientNonceMetric> {
        let matched = collections::filter(items, |candidate: &LenientNonceMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_nonce_metrics(
    store: Rc<dyn LenientNonceMetricStore>,
    query: &LenientNonceMetricQuery,
) -> Vec<LenientNonceMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientNonceMetricStore> = new_memory_lenient_nonce_metric_store();
        let item = new_lenient_nonce_metric("code-603", "title-100");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
