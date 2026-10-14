use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global rollout variances by code.
pub trait GlobalRolloutVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalRolloutVariance>;
    fn save(&self, item: &GlobalRolloutVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalRolloutVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalRolloutVarianceStore {
    pub items: RefCell<BTreeMap<String, GlobalRolloutVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalRolloutVarianceStore for MemoryGlobalRolloutVarianceStore {
    fn find(&self, code: &str) -> Option<GlobalRolloutVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalRolloutVariance) {
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

    fn list(&self) -> Vec<GlobalRolloutVariance> {
        let mut out: Vec<GlobalRolloutVariance> = Vec::new();
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

pub fn new_memory_global_rollout_variance_store() -> Rc<dyn GlobalRolloutVarianceStore> {
    Rc::new(MemoryGlobalRolloutVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalRolloutVarianceStore {
    pub inner: Rc<dyn GlobalRolloutVarianceStore>,
    pub cache: RefCell<cache::TtlCache<GlobalRolloutVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalRolloutVarianceStore for CachedGlobalRolloutVarianceStore {
    fn find(&self, code: &str) -> Option<GlobalRolloutVariance> {
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

    fn save(&self, item: &GlobalRolloutVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalRolloutVariance> {
        self.inner.list()
    }
}

pub fn new_cached_global_rollout_variance_store(
    inner: Rc<dyn GlobalRolloutVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalRolloutVarianceStore> {
    Rc::new(CachedGlobalRolloutVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalRolloutVariance>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global rollout variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutVarianceQuery {
    pub label: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalRolloutVarianceQuery {
    pub fn matches(&self, item: &GlobalRolloutVariance) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalRolloutVariance]) -> Vec<GlobalRolloutVariance> {
        let matched = collections::filter(items, |candidate: &GlobalRolloutVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_rollout_variances(
    store: Rc<dyn GlobalRolloutVarianceStore>,
    query: &GlobalRolloutVarianceQuery,
) -> Vec<GlobalRolloutVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalRolloutVarianceStore> = new_memory_global_rollout_variance_store();
        let item = new_global_rollout_variance("code-445", "label-525");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
