use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal ttl policy estimates by code.
pub trait SeasonalTtlPolicyEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalTtlPolicyEstimate>;
    fn save(&self, item: &SeasonalTtlPolicyEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalTtlPolicyEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalTtlPolicyEstimateStore {
    pub items: RefCell<BTreeMap<String, SeasonalTtlPolicyEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalTtlPolicyEstimateStore for MemorySeasonalTtlPolicyEstimateStore {
    fn find(&self, code: &str) -> Option<SeasonalTtlPolicyEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalTtlPolicyEstimate) {
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

    fn list(&self) -> Vec<SeasonalTtlPolicyEstimate> {
        let mut out: Vec<SeasonalTtlPolicyEstimate> = Vec::new();
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

pub fn new_memory_seasonal_ttl_policy_estimate_store() -> Rc<dyn SeasonalTtlPolicyEstimateStore> {
    Rc::new(MemorySeasonalTtlPolicyEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalTtlPolicyEstimateStore {
    pub inner: Rc<dyn SeasonalTtlPolicyEstimateStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalTtlPolicyEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalTtlPolicyEstimateStore for CachedSeasonalTtlPolicyEstimateStore {
    fn find(&self, code: &str) -> Option<SeasonalTtlPolicyEstimate> {
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

    fn save(&self, item: &SeasonalTtlPolicyEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalTtlPolicyEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_ttl_policy_estimate_store(
    inner: Rc<dyn SeasonalTtlPolicyEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalTtlPolicyEstimateStore> {
    Rc::new(CachedSeasonalTtlPolicyEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalTtlPolicyEstimate>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal ttl policy estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEstimateQuery {
    pub country: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalTtlPolicyEstimateQuery {
    pub fn matches(&self, item: &SeasonalTtlPolicyEstimate) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalTtlPolicyEstimate]) -> Vec<SeasonalTtlPolicyEstimate> {
        let matched = collections::filter(items, |candidate: &SeasonalTtlPolicyEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_ttl_policy_estimates(
    store: Rc<dyn SeasonalTtlPolicyEstimateStore>,
    query: &SeasonalTtlPolicyEstimateQuery,
) -> Vec<SeasonalTtlPolicyEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalTtlPolicyEstimateStore> = new_memory_seasonal_ttl_policy_estimate_store();
        let item = new_seasonal_ttl_policy_estimate("code-6", "label-568");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
