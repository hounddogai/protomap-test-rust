use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted federation estimates by code.
pub trait WeightedFederationEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedFederationEstimate>;
    fn save(&self, item: &WeightedFederationEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedFederationEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedFederationEstimateStore {
    pub items: RefCell<BTreeMap<String, WeightedFederationEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedFederationEstimateStore for MemoryWeightedFederationEstimateStore {
    fn find(&self, code: &str) -> Option<WeightedFederationEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedFederationEstimate) {
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

    fn list(&self) -> Vec<WeightedFederationEstimate> {
        let mut out: Vec<WeightedFederationEstimate> = Vec::new();
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

pub fn new_memory_weighted_federation_estimate_store() -> Rc<dyn WeightedFederationEstimateStore> {
    Rc::new(MemoryWeightedFederationEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedFederationEstimateStore {
    pub inner: Rc<dyn WeightedFederationEstimateStore>,
    pub cache: RefCell<cache::TtlCache<WeightedFederationEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedFederationEstimateStore for CachedWeightedFederationEstimateStore {
    fn find(&self, code: &str) -> Option<WeightedFederationEstimate> {
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

    fn save(&self, item: &WeightedFederationEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedFederationEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_federation_estimate_store(
    inner: Rc<dyn WeightedFederationEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedFederationEstimateStore> {
    Rc::new(CachedWeightedFederationEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedFederationEstimate>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted federation estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationEstimateQuery {
    pub country: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedFederationEstimateQuery {
    pub fn matches(&self, item: &WeightedFederationEstimate) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedFederationEstimate]) -> Vec<WeightedFederationEstimate> {
        let matched = collections::filter(items, |candidate: &WeightedFederationEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_federation_estimates(
    store: Rc<dyn WeightedFederationEstimateStore>,
    query: &WeightedFederationEstimateQuery,
) -> Vec<WeightedFederationEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedFederationEstimateStore> = new_memory_weighted_federation_estimate_store();
        let item = new_weighted_federation_estimate("code-596", "label-296");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
