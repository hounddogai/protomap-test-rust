use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual tenant claims by code.
pub trait ManualTenantClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualTenantClaim>;
    fn save(&self, item: &ManualTenantClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualTenantClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualTenantClaimStore {
    pub items: RefCell<BTreeMap<String, ManualTenantClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualTenantClaimStore for MemoryManualTenantClaimStore {
    fn find(&self, code: &str) -> Option<ManualTenantClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualTenantClaim) {
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

    fn list(&self) -> Vec<ManualTenantClaim> {
        let mut out: Vec<ManualTenantClaim> = Vec::new();
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

pub fn new_memory_manual_tenant_claim_store() -> Rc<dyn ManualTenantClaimStore> {
    Rc::new(MemoryManualTenantClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualTenantClaimStore {
    pub inner: Rc<dyn ManualTenantClaimStore>,
    pub cache: RefCell<cache::TtlCache<ManualTenantClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualTenantClaimStore for CachedManualTenantClaimStore {
    fn find(&self, code: &str) -> Option<ManualTenantClaim> {
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

    fn save(&self, item: &ManualTenantClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualTenantClaim> {
        self.inner.list()
    }
}

pub fn new_cached_manual_tenant_claim_store(
    inner: Rc<dyn ManualTenantClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualTenantClaimStore> {
    Rc::new(CachedManualTenantClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualTenantClaim>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual tenant claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantClaimQuery {
    pub title: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualTenantClaimQuery {
    pub fn matches(&self, item: &ManualTenantClaim) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualTenantClaim]) -> Vec<ManualTenantClaim> {
        let matched = collections::filter(items, |candidate: &ManualTenantClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_tenant_claims(
    store: Rc<dyn ManualTenantClaimStore>,
    query: &ManualTenantClaimQuery,
) -> Vec<ManualTenantClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualTenantClaimStore> = new_memory_manual_tenant_claim_store();
        let item = new_manual_tenant_claim("code-709", "title-729");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
