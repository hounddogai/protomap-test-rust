use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential thresholds by code.
pub trait CredentialThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialThreshold>;
    fn save(&self, item: &CredentialThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialThresholdStore {
    pub items: RefCell<BTreeMap<String, CredentialThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialThresholdStore for MemoryCredentialThresholdStore {
    fn find(&self, code: &str) -> Option<CredentialThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialThreshold) {
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

    fn list(&self) -> Vec<CredentialThreshold> {
        let mut out: Vec<CredentialThreshold> = Vec::new();
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

pub fn new_memory_credential_threshold_store() -> Rc<dyn CredentialThresholdStore> {
    Rc::new(MemoryCredentialThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialThresholdStore {
    pub inner: Rc<dyn CredentialThresholdStore>,
    pub cache: RefCell<cache::TtlCache<CredentialThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialThresholdStore for CachedCredentialThresholdStore {
    fn find(&self, code: &str) -> Option<CredentialThreshold> {
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

    fn save(&self, item: &CredentialThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_credential_threshold_store(
    inner: Rc<dyn CredentialThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialThresholdStore> {
    Rc::new(CachedCredentialThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialThreshold>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialThresholdQuery {
    pub title: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialThresholdQuery {
    pub fn matches(&self, item: &CredentialThreshold) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[CredentialThreshold]) -> Vec<CredentialThreshold> {
        let matched = collections::filter(items, |candidate: &CredentialThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_thresholds(
    store: Rc<dyn CredentialThresholdStore>,
    query: &CredentialThresholdQuery,
) -> Vec<CredentialThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CredentialThresholdStore> = new_memory_credential_threshold_store();
        let item = new_credential_threshold("code-342", "title-956");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
