use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered identity metrics by code.
pub trait TieredIdentityMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredIdentityMetric>;
    fn save(&self, item: &TieredIdentityMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredIdentityMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredIdentityMetricStore {
    pub items: RefCell<BTreeMap<String, TieredIdentityMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredIdentityMetricStore for MemoryTieredIdentityMetricStore {
    fn find(&self, code: &str) -> Option<TieredIdentityMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredIdentityMetric) {
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

    fn list(&self) -> Vec<TieredIdentityMetric> {
        let mut out: Vec<TieredIdentityMetric> = Vec::new();
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

pub fn new_memory_tiered_identity_metric_store() -> Rc<dyn TieredIdentityMetricStore> {
    Rc::new(MemoryTieredIdentityMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredIdentityMetricStore {
    pub inner: Rc<dyn TieredIdentityMetricStore>,
    pub cache: RefCell<cache::TtlCache<TieredIdentityMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredIdentityMetricStore for CachedTieredIdentityMetricStore {
    fn find(&self, code: &str) -> Option<TieredIdentityMetric> {
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

    fn save(&self, item: &TieredIdentityMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredIdentityMetric> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_identity_metric_store(
    inner: Rc<dyn TieredIdentityMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredIdentityMetricStore> {
    Rc::new(CachedTieredIdentityMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredIdentityMetric>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered identity metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityMetricQuery {
    pub segment: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredIdentityMetricQuery {
    pub fn matches(&self, item: &TieredIdentityMetric) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredIdentityMetric]) -> Vec<TieredIdentityMetric> {
        let matched = collections::filter(items, |candidate: &TieredIdentityMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_identity_metrics(
    store: Rc<dyn TieredIdentityMetricStore>,
    query: &TieredIdentityMetricQuery,
) -> Vec<TieredIdentityMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredIdentityMetricStore> = new_memory_tiered_identity_metric_store();
        let item = new_tiered_identity_metric("code-299", "name-409");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
