use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual region pin policies by code.
pub trait ManualRegionPinPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualRegionPinPolicy>;
    fn save(&self, item: &ManualRegionPinPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualRegionPinPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualRegionPinPolicyStore {
    pub items: RefCell<BTreeMap<String, ManualRegionPinPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualRegionPinPolicyStore for MemoryManualRegionPinPolicyStore {
    fn find(&self, code: &str) -> Option<ManualRegionPinPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualRegionPinPolicy) {
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

    fn list(&self) -> Vec<ManualRegionPinPolicy> {
        let mut out: Vec<ManualRegionPinPolicy> = Vec::new();
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

pub fn new_memory_manual_region_pin_policy_store() -> Rc<dyn ManualRegionPinPolicyStore> {
    Rc::new(MemoryManualRegionPinPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualRegionPinPolicyStore {
    pub inner: Rc<dyn ManualRegionPinPolicyStore>,
    pub cache: RefCell<cache::TtlCache<ManualRegionPinPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualRegionPinPolicyStore for CachedManualRegionPinPolicyStore {
    fn find(&self, code: &str) -> Option<ManualRegionPinPolicy> {
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

    fn save(&self, item: &ManualRegionPinPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualRegionPinPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_manual_region_pin_policy_store(
    inner: Rc<dyn ManualRegionPinPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualRegionPinPolicyStore> {
    Rc::new(CachedManualRegionPinPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualRegionPinPolicy>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual region pin policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinPolicyQuery {
    pub category: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualRegionPinPolicyQuery {
    pub fn matches(&self, item: &ManualRegionPinPolicy) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualRegionPinPolicy]) -> Vec<ManualRegionPinPolicy> {
        let matched = collections::filter(items, |candidate: &ManualRegionPinPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_region_pin_policies(
    store: Rc<dyn ManualRegionPinPolicyStore>,
    query: &ManualRegionPinPolicyQuery,
) -> Vec<ManualRegionPinPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualRegionPinPolicyStore> = new_memory_manual_region_pin_policy_store();
        let item = new_manual_region_pin_policy("code-143", "title-29");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
