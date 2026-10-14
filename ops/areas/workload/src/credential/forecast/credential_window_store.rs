use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential windows by code.
pub trait CredentialWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialWindow>;
    fn save(&self, item: &CredentialWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialWindowStore {
    pub items: RefCell<BTreeMap<String, CredentialWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialWindowStore for MemoryCredentialWindowStore {
    fn find(&self, code: &str) -> Option<CredentialWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialWindow) {
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

    fn list(&self) -> Vec<CredentialWindow> {
        let mut out: Vec<CredentialWindow> = Vec::new();
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

pub fn new_memory_credential_window_store() -> Rc<dyn CredentialWindowStore> {
    Rc::new(MemoryCredentialWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialWindowStore {
    pub inner: Rc<dyn CredentialWindowStore>,
    pub cache: RefCell<cache::TtlCache<CredentialWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialWindowStore for CachedCredentialWindowStore {
    fn find(&self, code: &str) -> Option<CredentialWindow> {
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

    fn save(&self, item: &CredentialWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialWindow> {
        self.inner.list()
    }
}

pub fn new_cached_credential_window_store(
    inner: Rc<dyn CredentialWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialWindowStore> {
    Rc::new(CachedCredentialWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialWindow>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialWindowQuery {
    pub region: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialWindowQuery {
    pub fn matches(&self, item: &CredentialWindow) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CredentialWindow]) -> Vec<CredentialWindow> {
        let matched = collections::filter(items, |candidate: &CredentialWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_windows(
    store: Rc<dyn CredentialWindowStore>,
    query: &CredentialWindowQuery,
) -> Vec<CredentialWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CredentialWindowStore> = new_memory_credential_window_store();
        let item = new_credential_window("code-197", "name-884");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
