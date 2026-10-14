use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential groups by code.
pub trait CredentialGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialGroup>;
    fn save(&self, item: &CredentialGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialGroupStore {
    pub items: RefCell<BTreeMap<String, CredentialGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialGroupStore for MemoryCredentialGroupStore {
    fn find(&self, code: &str) -> Option<CredentialGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialGroup) {
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

    fn list(&self) -> Vec<CredentialGroup> {
        let mut out: Vec<CredentialGroup> = Vec::new();
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

pub fn new_memory_credential_group_store() -> Rc<dyn CredentialGroupStore> {
    Rc::new(MemoryCredentialGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialGroupStore {
    pub inner: Rc<dyn CredentialGroupStore>,
    pub cache: RefCell<cache::TtlCache<CredentialGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialGroupStore for CachedCredentialGroupStore {
    fn find(&self, code: &str) -> Option<CredentialGroup> {
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

    fn save(&self, item: &CredentialGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialGroup> {
        self.inner.list()
    }
}

pub fn new_cached_credential_group_store(
    inner: Rc<dyn CredentialGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialGroupStore> {
    Rc::new(CachedCredentialGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialGroup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialGroupQuery {
    pub unit: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialGroupQuery {
    pub fn matches(&self, item: &CredentialGroup) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CredentialGroup]) -> Vec<CredentialGroup> {
        let matched = collections::filter(items, |candidate: &CredentialGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_groups(
    store: Rc<dyn CredentialGroupStore>,
    query: &CredentialGroupQuery,
) -> Vec<CredentialGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CredentialGroupStore> = new_memory_credential_group_store();
        let item = new_credential_group("code-783", "label-725");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
