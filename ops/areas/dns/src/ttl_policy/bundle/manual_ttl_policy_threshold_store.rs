use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual ttl policy thresholds by code.
pub trait ManualTtlPolicyThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualTtlPolicyThreshold>;
    fn save(&self, item: &ManualTtlPolicyThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualTtlPolicyThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualTtlPolicyThresholdStore {
    pub items: RefCell<BTreeMap<String, ManualTtlPolicyThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualTtlPolicyThresholdStore for MemoryManualTtlPolicyThresholdStore {
    fn find(&self, code: &str) -> Option<ManualTtlPolicyThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualTtlPolicyThreshold) {
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

    fn list(&self) -> Vec<ManualTtlPolicyThreshold> {
        let mut out: Vec<ManualTtlPolicyThreshold> = Vec::new();
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

pub fn new_memory_manual_ttl_policy_threshold_store() -> Rc<dyn ManualTtlPolicyThresholdStore> {
    Rc::new(MemoryManualTtlPolicyThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualTtlPolicyThresholdStore {
    pub inner: Rc<dyn ManualTtlPolicyThresholdStore>,
    pub cache: RefCell<cache::TtlCache<ManualTtlPolicyThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualTtlPolicyThresholdStore for CachedManualTtlPolicyThresholdStore {
    fn find(&self, code: &str) -> Option<ManualTtlPolicyThreshold> {
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

    fn save(&self, item: &ManualTtlPolicyThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualTtlPolicyThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_manual_ttl_policy_threshold_store(
    inner: Rc<dyn ManualTtlPolicyThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualTtlPolicyThresholdStore> {
    Rc::new(CachedManualTtlPolicyThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualTtlPolicyThreshold>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual ttl policy thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyThresholdQuery {
    pub locale: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualTtlPolicyThresholdQuery {
    pub fn matches(&self, item: &ManualTtlPolicyThreshold) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualTtlPolicyThreshold]) -> Vec<ManualTtlPolicyThreshold> {
        let matched = collections::filter(items, |candidate: &ManualTtlPolicyThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_ttl_policy_thresholds(
    store: Rc<dyn ManualTtlPolicyThresholdStore>,
    query: &ManualTtlPolicyThresholdQuery,
) -> Vec<ManualTtlPolicyThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualTtlPolicyThresholdStore> = new_memory_manual_ttl_policy_threshold_store();
        let item = new_manual_ttl_policy_threshold("code-942", "title-24");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
