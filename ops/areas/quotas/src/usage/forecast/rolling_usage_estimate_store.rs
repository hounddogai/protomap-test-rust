use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling usage estimates by code.
pub trait RollingUsageEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingUsageEstimate>;
    fn save(&self, item: &RollingUsageEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingUsageEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingUsageEstimateStore {
    pub items: RefCell<BTreeMap<String, RollingUsageEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingUsageEstimateStore for MemoryRollingUsageEstimateStore {
    fn find(&self, code: &str) -> Option<RollingUsageEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingUsageEstimate) {
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

    fn list(&self) -> Vec<RollingUsageEstimate> {
        let mut out: Vec<RollingUsageEstimate> = Vec::new();
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

pub fn new_memory_rolling_usage_estimate_store() -> Rc<dyn RollingUsageEstimateStore> {
    Rc::new(MemoryRollingUsageEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingUsageEstimateStore {
    pub inner: Rc<dyn RollingUsageEstimateStore>,
    pub cache: RefCell<cache::TtlCache<RollingUsageEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingUsageEstimateStore for CachedRollingUsageEstimateStore {
    fn find(&self, code: &str) -> Option<RollingUsageEstimate> {
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

    fn save(&self, item: &RollingUsageEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingUsageEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_usage_estimate_store(
    inner: Rc<dyn RollingUsageEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingUsageEstimateStore> {
    Rc::new(CachedRollingUsageEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingUsageEstimate>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling usage estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageEstimateQuery {
    pub region: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingUsageEstimateQuery {
    pub fn matches(&self, item: &RollingUsageEstimate) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[RollingUsageEstimate]) -> Vec<RollingUsageEstimate> {
        let matched = collections::filter(items, |candidate: &RollingUsageEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_usage_estimates(
    store: Rc<dyn RollingUsageEstimateStore>,
    query: &RollingUsageEstimateQuery,
) -> Vec<RollingUsageEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingUsageEstimateStore> = new_memory_rolling_usage_estimate_store();
        let item = new_rolling_usage_estimate("code-142", "label-465");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
