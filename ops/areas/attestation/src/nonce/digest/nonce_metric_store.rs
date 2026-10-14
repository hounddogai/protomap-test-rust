use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps nonce metrics by code.
pub trait NonceMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<NonceMetric>;
    fn save(&self, item: &NonceMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<NonceMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNonceMetricStore {
    pub items: RefCell<BTreeMap<String, NonceMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl NonceMetricStore for MemoryNonceMetricStore {
    fn find(&self, code: &str) -> Option<NonceMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &NonceMetric) {
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

    fn list(&self) -> Vec<NonceMetric> {
        let mut out: Vec<NonceMetric> = Vec::new();
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

pub fn new_memory_nonce_metric_store() -> Rc<dyn NonceMetricStore> {
    Rc::new(MemoryNonceMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedNonceMetricStore {
    pub inner: Rc<dyn NonceMetricStore>,
    pub cache: RefCell<cache::TtlCache<NonceMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl NonceMetricStore for CachedNonceMetricStore {
    fn find(&self, code: &str) -> Option<NonceMetric> {
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

    fn save(&self, item: &NonceMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<NonceMetric> {
        self.inner.list()
    }
}

pub fn new_cached_nonce_metric_store(
    inner: Rc<dyn NonceMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn NonceMetricStore> {
    Rc::new(CachedNonceMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<NonceMetric>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects nonce metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceMetricQuery {
    pub title: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl NonceMetricQuery {
    pub fn matches(&self, item: &NonceMetric) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[NonceMetric]) -> Vec<NonceMetric> {
        let matched = collections::filter(items, |candidate: &NonceMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_nonce_metrics(store: Rc<dyn NonceMetricStore>, query: &NonceMetricQuery) -> Vec<NonceMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn NonceMetricStore> = new_memory_nonce_metric_store();
        let item = new_nonce_metric("code-549", "title-653");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
