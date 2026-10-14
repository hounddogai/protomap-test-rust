use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted rollback bundles by code.
pub trait WeightedRollbackBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedRollbackBundle>;
    fn save(&self, item: &WeightedRollbackBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedRollbackBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedRollbackBundleStore {
    pub items: RefCell<BTreeMap<String, WeightedRollbackBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedRollbackBundleStore for MemoryWeightedRollbackBundleStore {
    fn find(&self, code: &str) -> Option<WeightedRollbackBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedRollbackBundle) {
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

    fn list(&self) -> Vec<WeightedRollbackBundle> {
        let mut out: Vec<WeightedRollbackBundle> = Vec::new();
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

pub fn new_memory_weighted_rollback_bundle_store() -> Rc<dyn WeightedRollbackBundleStore> {
    Rc::new(MemoryWeightedRollbackBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedRollbackBundleStore {
    pub inner: Rc<dyn WeightedRollbackBundleStore>,
    pub cache: RefCell<cache::TtlCache<WeightedRollbackBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedRollbackBundleStore for CachedWeightedRollbackBundleStore {
    fn find(&self, code: &str) -> Option<WeightedRollbackBundle> {
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

    fn save(&self, item: &WeightedRollbackBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedRollbackBundle> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_rollback_bundle_store(
    inner: Rc<dyn WeightedRollbackBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedRollbackBundleStore> {
    Rc::new(CachedWeightedRollbackBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedRollbackBundle>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted rollback bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBundleQuery {
    pub segment: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedRollbackBundleQuery {
    pub fn matches(&self, item: &WeightedRollbackBundle) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedRollbackBundle]) -> Vec<WeightedRollbackBundle> {
        let matched = collections::filter(items, |candidate: &WeightedRollbackBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_rollback_bundles(
    store: Rc<dyn WeightedRollbackBundleStore>,
    query: &WeightedRollbackBundleQuery,
) -> Vec<WeightedRollbackBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedRollbackBundleStore> = new_memory_weighted_rollback_bundle_store();
        let item = new_weighted_rollback_bundle("code-545", "name-9");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
