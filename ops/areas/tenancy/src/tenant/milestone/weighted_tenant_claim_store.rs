use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted tenant claims by code.
pub trait WeightedTenantClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedTenantClaim>;
    fn save(&self, item: &WeightedTenantClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedTenantClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedTenantClaimStore {
    pub items: RefCell<BTreeMap<String, WeightedTenantClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedTenantClaimStore for MemoryWeightedTenantClaimStore {
    fn find(&self, code: &str) -> Option<WeightedTenantClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedTenantClaim) {
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

    fn list(&self) -> Vec<WeightedTenantClaim> {
        let mut out: Vec<WeightedTenantClaim> = Vec::new();
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

pub fn new_memory_weighted_tenant_claim_store() -> Rc<dyn WeightedTenantClaimStore> {
    Rc::new(MemoryWeightedTenantClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedTenantClaimStore {
    pub inner: Rc<dyn WeightedTenantClaimStore>,
    pub cache: RefCell<cache::TtlCache<WeightedTenantClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedTenantClaimStore for CachedWeightedTenantClaimStore {
    fn find(&self, code: &str) -> Option<WeightedTenantClaim> {
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

    fn save(&self, item: &WeightedTenantClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedTenantClaim> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_tenant_claim_store(
    inner: Rc<dyn WeightedTenantClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedTenantClaimStore> {
    Rc::new(CachedWeightedTenantClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedTenantClaim>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted tenant claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTenantClaimQuery {
    pub namespace: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedTenantClaimQuery {
    pub fn matches(&self, item: &WeightedTenantClaim) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedTenantClaim]) -> Vec<WeightedTenantClaim> {
        let matched = collections::filter(items, |candidate: &WeightedTenantClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_tenant_claims(
    store: Rc<dyn WeightedTenantClaimStore>,
    query: &WeightedTenantClaimQuery,
) -> Vec<WeightedTenantClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedTenantClaimStore> = new_memory_weighted_tenant_claim_store();
        let item = new_weighted_tenant_claim("code-31", "label-701");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
