use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual retention bundles by code.
pub trait ManualRetentionBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualRetentionBundle>;
    fn save(&self, item: &ManualRetentionBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualRetentionBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualRetentionBundleStore {
    pub items: RefCell<BTreeMap<String, ManualRetentionBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualRetentionBundleStore for MemoryManualRetentionBundleStore {
    fn find(&self, code: &str) -> Option<ManualRetentionBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualRetentionBundle) {
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

    fn list(&self) -> Vec<ManualRetentionBundle> {
        let mut out: Vec<ManualRetentionBundle> = Vec::new();
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

pub fn new_memory_manual_retention_bundle_store() -> Rc<dyn ManualRetentionBundleStore> {
    Rc::new(MemoryManualRetentionBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualRetentionBundleStore {
    pub inner: Rc<dyn ManualRetentionBundleStore>,
    pub cache: RefCell<cache::TtlCache<ManualRetentionBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualRetentionBundleStore for CachedManualRetentionBundleStore {
    fn find(&self, code: &str) -> Option<ManualRetentionBundle> {
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

    fn save(&self, item: &ManualRetentionBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualRetentionBundle> {
        self.inner.list()
    }
}

pub fn new_cached_manual_retention_bundle_store(
    inner: Rc<dyn ManualRetentionBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualRetentionBundleStore> {
    Rc::new(CachedManualRetentionBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualRetentionBundle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual retention bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionBundleQuery {
    pub region: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualRetentionBundleQuery {
    pub fn matches(&self, item: &ManualRetentionBundle) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[ManualRetentionBundle]) -> Vec<ManualRetentionBundle> {
        let matched = collections::filter(items, |candidate: &ManualRetentionBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_retention_bundles(
    store: Rc<dyn ManualRetentionBundleStore>,
    query: &ManualRetentionBundleQuery,
) -> Vec<ManualRetentionBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualRetentionBundleStore> = new_memory_manual_retention_bundle_store();
        let item = new_manual_retention_bundle("code-733", "label-639");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
