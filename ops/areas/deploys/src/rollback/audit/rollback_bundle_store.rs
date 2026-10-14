use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback bundles by code.
pub trait RollbackBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackBundle>;
    fn save(&self, item: &RollbackBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackBundleStore {
    pub items: RefCell<BTreeMap<String, RollbackBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackBundleStore for MemoryRollbackBundleStore {
    fn find(&self, code: &str) -> Option<RollbackBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackBundle) {
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

    fn list(&self) -> Vec<RollbackBundle> {
        let mut out: Vec<RollbackBundle> = Vec::new();
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

pub fn new_memory_rollback_bundle_store() -> Rc<dyn RollbackBundleStore> {
    Rc::new(MemoryRollbackBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackBundleStore {
    pub inner: Rc<dyn RollbackBundleStore>,
    pub cache: RefCell<cache::TtlCache<RollbackBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackBundleStore for CachedRollbackBundleStore {
    fn find(&self, code: &str) -> Option<RollbackBundle> {
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

    fn save(&self, item: &RollbackBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackBundle> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_bundle_store(
    inner: Rc<dyn RollbackBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackBundleStore> {
    Rc::new(CachedRollbackBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackBundle>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBundleQuery {
    pub channel: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackBundleQuery {
    pub fn matches(&self, item: &RollbackBundle) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollbackBundle]) -> Vec<RollbackBundle> {
        let matched = collections::filter(items, |candidate: &RollbackBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_bundles(store: Rc<dyn RollbackBundleStore>, query: &RollbackBundleQuery) -> Vec<RollbackBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollbackBundleStore> = new_memory_rollback_bundle_store();
        let item = new_rollback_bundle("code-189", "name-326");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
