use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual credential bundles by code.
pub trait ManualCredentialBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualCredentialBundle>;
    fn save(&self, item: &ManualCredentialBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualCredentialBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualCredentialBundleStore {
    pub items: RefCell<BTreeMap<String, ManualCredentialBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualCredentialBundleStore for MemoryManualCredentialBundleStore {
    fn find(&self, code: &str) -> Option<ManualCredentialBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualCredentialBundle) {
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

    fn list(&self) -> Vec<ManualCredentialBundle> {
        let mut out: Vec<ManualCredentialBundle> = Vec::new();
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

pub fn new_memory_manual_credential_bundle_store() -> Rc<dyn ManualCredentialBundleStore> {
    Rc::new(MemoryManualCredentialBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualCredentialBundleStore {
    pub inner: Rc<dyn ManualCredentialBundleStore>,
    pub cache: RefCell<cache::TtlCache<ManualCredentialBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualCredentialBundleStore for CachedManualCredentialBundleStore {
    fn find(&self, code: &str) -> Option<ManualCredentialBundle> {
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

    fn save(&self, item: &ManualCredentialBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualCredentialBundle> {
        self.inner.list()
    }
}

pub fn new_cached_manual_credential_bundle_store(
    inner: Rc<dyn ManualCredentialBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualCredentialBundleStore> {
    Rc::new(CachedManualCredentialBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualCredentialBundle>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual credential bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialBundleQuery {
    pub title: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualCredentialBundleQuery {
    pub fn matches(&self, item: &ManualCredentialBundle) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualCredentialBundle]) -> Vec<ManualCredentialBundle> {
        let matched = collections::filter(items, |candidate: &ManualCredentialBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_credential_bundles(
    store: Rc<dyn ManualCredentialBundleStore>,
    query: &ManualCredentialBundleQuery,
) -> Vec<ManualCredentialBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualCredentialBundleStore> = new_memory_manual_credential_bundle_store();
        let item = new_manual_credential_bundle("code-380", "title-47");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
