use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive forecast policies by code.
pub trait AdaptiveForecastPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveForecastPolicy>;
    fn save(&self, item: &AdaptiveForecastPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveForecastPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveForecastPolicyStore {
    pub items: RefCell<BTreeMap<String, AdaptiveForecastPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveForecastPolicyStore for MemoryAdaptiveForecastPolicyStore {
    fn find(&self, code: &str) -> Option<AdaptiveForecastPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveForecastPolicy) {
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

    fn list(&self) -> Vec<AdaptiveForecastPolicy> {
        let mut out: Vec<AdaptiveForecastPolicy> = Vec::new();
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

pub fn new_memory_adaptive_forecast_policy_store() -> Rc<dyn AdaptiveForecastPolicyStore> {
    Rc::new(MemoryAdaptiveForecastPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveForecastPolicyStore {
    pub inner: Rc<dyn AdaptiveForecastPolicyStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveForecastPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveForecastPolicyStore for CachedAdaptiveForecastPolicyStore {
    fn find(&self, code: &str) -> Option<AdaptiveForecastPolicy> {
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

    fn save(&self, item: &AdaptiveForecastPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveForecastPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_forecast_policy_store(
    inner: Rc<dyn AdaptiveForecastPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveForecastPolicyStore> {
    Rc::new(CachedAdaptiveForecastPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveForecastPolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive forecast policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastPolicyQuery {
    pub country: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveForecastPolicyQuery {
    pub fn matches(&self, item: &AdaptiveForecastPolicy) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveForecastPolicy]) -> Vec<AdaptiveForecastPolicy> {
        let matched = collections::filter(items, |candidate: &AdaptiveForecastPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_forecast_policies(
    store: Rc<dyn AdaptiveForecastPolicyStore>,
    query: &AdaptiveForecastPolicyQuery,
) -> Vec<AdaptiveForecastPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveForecastPolicyStore> = new_memory_adaptive_forecast_policy_store();
        let item = new_adaptive_forecast_policy("code-21", "title-221");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
