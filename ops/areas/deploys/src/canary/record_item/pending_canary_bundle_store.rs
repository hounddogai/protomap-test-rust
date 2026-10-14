use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending canary bundles by code.
pub trait PendingCanaryBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingCanaryBundle>;
    fn save(&self, item: &PendingCanaryBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingCanaryBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingCanaryBundleStore {
    pub items: RefCell<BTreeMap<String, PendingCanaryBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingCanaryBundleStore for MemoryPendingCanaryBundleStore {
    fn find(&self, code: &str) -> Option<PendingCanaryBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingCanaryBundle) {
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

    fn list(&self) -> Vec<PendingCanaryBundle> {
        let mut out: Vec<PendingCanaryBundle> = Vec::new();
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

pub fn new_memory_pending_canary_bundle_store() -> Rc<dyn PendingCanaryBundleStore> {
    Rc::new(MemoryPendingCanaryBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingCanaryBundleStore {
    pub inner: Rc<dyn PendingCanaryBundleStore>,
    pub cache: RefCell<cache::TtlCache<PendingCanaryBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingCanaryBundleStore for CachedPendingCanaryBundleStore {
    fn find(&self, code: &str) -> Option<PendingCanaryBundle> {
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

    fn save(&self, item: &PendingCanaryBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingCanaryBundle> {
        self.inner.list()
    }
}

pub fn new_cached_pending_canary_bundle_store(
    inner: Rc<dyn PendingCanaryBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingCanaryBundleStore> {
    Rc::new(CachedPendingCanaryBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingCanaryBundle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending canary bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryBundleQuery {
    pub locale: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingCanaryBundleQuery {
    pub fn matches(&self, item: &PendingCanaryBundle) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingCanaryBundle]) -> Vec<PendingCanaryBundle> {
        let matched = collections::filter(items, |candidate: &PendingCanaryBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_canary_bundles(
    store: Rc<dyn PendingCanaryBundleStore>,
    query: &PendingCanaryBundleQuery,
) -> Vec<PendingCanaryBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingCanaryBundleStore> = new_memory_pending_canary_bundle_store();
        let item = new_pending_canary_bundle("code-846", "label-744");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
