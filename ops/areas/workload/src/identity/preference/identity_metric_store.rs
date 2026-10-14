use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity metrics by code.
pub trait IdentityMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityMetric>;
    fn save(&self, item: &IdentityMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityMetricStore {
    pub items: RefCell<BTreeMap<String, IdentityMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityMetricStore for MemoryIdentityMetricStore {
    fn find(&self, code: &str) -> Option<IdentityMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityMetric) {
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

    fn list(&self) -> Vec<IdentityMetric> {
        let mut out: Vec<IdentityMetric> = Vec::new();
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

pub fn new_memory_identity_metric_store() -> Rc<dyn IdentityMetricStore> {
    Rc::new(MemoryIdentityMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityMetricStore {
    pub inner: Rc<dyn IdentityMetricStore>,
    pub cache: RefCell<cache::TtlCache<IdentityMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityMetricStore for CachedIdentityMetricStore {
    fn find(&self, code: &str) -> Option<IdentityMetric> {
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

    fn save(&self, item: &IdentityMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityMetric> {
        self.inner.list()
    }
}

pub fn new_cached_identity_metric_store(
    inner: Rc<dyn IdentityMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityMetricStore> {
    Rc::new(CachedIdentityMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityMetric>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMetricQuery {
    pub country: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityMetricQuery {
    pub fn matches(&self, item: &IdentityMetric) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[IdentityMetric]) -> Vec<IdentityMetric> {
        let matched = collections::filter(items, |candidate: &IdentityMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_metrics(store: Rc<dyn IdentityMetricStore>, query: &IdentityMetricQuery) -> Vec<IdentityMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IdentityMetricStore> = new_memory_identity_metric_store();
        let item = new_identity_metric("code-109", "label-464");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
