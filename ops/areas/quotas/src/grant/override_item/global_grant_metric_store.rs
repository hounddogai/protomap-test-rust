use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global grant metrics by code.
pub trait GlobalGrantMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalGrantMetric>;
    fn save(&self, item: &GlobalGrantMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalGrantMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalGrantMetricStore {
    pub items: RefCell<BTreeMap<String, GlobalGrantMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalGrantMetricStore for MemoryGlobalGrantMetricStore {
    fn find(&self, code: &str) -> Option<GlobalGrantMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalGrantMetric) {
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

    fn list(&self) -> Vec<GlobalGrantMetric> {
        let mut out: Vec<GlobalGrantMetric> = Vec::new();
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

pub fn new_memory_global_grant_metric_store() -> Rc<dyn GlobalGrantMetricStore> {
    Rc::new(MemoryGlobalGrantMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalGrantMetricStore {
    pub inner: Rc<dyn GlobalGrantMetricStore>,
    pub cache: RefCell<cache::TtlCache<GlobalGrantMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalGrantMetricStore for CachedGlobalGrantMetricStore {
    fn find(&self, code: &str) -> Option<GlobalGrantMetric> {
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

    fn save(&self, item: &GlobalGrantMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalGrantMetric> {
        self.inner.list()
    }
}

pub fn new_cached_global_grant_metric_store(
    inner: Rc<dyn GlobalGrantMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalGrantMetricStore> {
    Rc::new(CachedGlobalGrantMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalGrantMetric>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global grant metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalGrantMetricQuery {
    pub segment: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalGrantMetricQuery {
    pub fn matches(&self, item: &GlobalGrantMetric) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalGrantMetric]) -> Vec<GlobalGrantMetric> {
        let matched = collections::filter(items, |candidate: &GlobalGrantMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_grant_metrics(
    store: Rc<dyn GlobalGrantMetricStore>,
    query: &GlobalGrantMetricQuery,
) -> Vec<GlobalGrantMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalGrantMetricStore> = new_memory_global_grant_metric_store();
        let item = new_global_grant_metric("code-397", "name-491");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
