use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential claims by code.
pub trait CredentialClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialClaim>;
    fn save(&self, item: &CredentialClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialClaimStore {
    pub items: RefCell<BTreeMap<String, CredentialClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialClaimStore for MemoryCredentialClaimStore {
    fn find(&self, code: &str) -> Option<CredentialClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialClaim) {
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

    fn list(&self) -> Vec<CredentialClaim> {
        let mut out: Vec<CredentialClaim> = Vec::new();
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

pub fn new_memory_credential_claim_store() -> Rc<dyn CredentialClaimStore> {
    Rc::new(MemoryCredentialClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialClaimStore {
    pub inner: Rc<dyn CredentialClaimStore>,
    pub cache: RefCell<cache::TtlCache<CredentialClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialClaimStore for CachedCredentialClaimStore {
    fn find(&self, code: &str) -> Option<CredentialClaim> {
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

    fn save(&self, item: &CredentialClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialClaim> {
        self.inner.list()
    }
}

pub fn new_cached_credential_claim_store(
    inner: Rc<dyn CredentialClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialClaimStore> {
    Rc::new(CachedCredentialClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialClaim>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialClaimQuery {
    pub name: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialClaimQuery {
    pub fn matches(&self, item: &CredentialClaim) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[CredentialClaim]) -> Vec<CredentialClaim> {
        let matched = collections::filter(items, |candidate: &CredentialClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_claims(
    store: Rc<dyn CredentialClaimStore>,
    query: &CredentialClaimQuery,
) -> Vec<CredentialClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CredentialClaimStore> = new_memory_credential_claim_store();
        let item = new_credential_claim("code-117", "name-526");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
