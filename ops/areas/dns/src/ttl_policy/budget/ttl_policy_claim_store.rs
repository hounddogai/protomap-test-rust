use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps ttl policy claims by code.
pub trait TtlPolicyClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TtlPolicyClaim>;
    fn save(&self, item: &TtlPolicyClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TtlPolicyClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTtlPolicyClaimStore {
    pub items: RefCell<BTreeMap<String, TtlPolicyClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl TtlPolicyClaimStore for MemoryTtlPolicyClaimStore {
    fn find(&self, code: &str) -> Option<TtlPolicyClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TtlPolicyClaim) {
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

    fn list(&self) -> Vec<TtlPolicyClaim> {
        let mut out: Vec<TtlPolicyClaim> = Vec::new();
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

pub fn new_memory_ttl_policy_claim_store() -> Rc<dyn TtlPolicyClaimStore> {
    Rc::new(MemoryTtlPolicyClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTtlPolicyClaimStore {
    pub inner: Rc<dyn TtlPolicyClaimStore>,
    pub cache: RefCell<cache::TtlCache<TtlPolicyClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TtlPolicyClaimStore for CachedTtlPolicyClaimStore {
    fn find(&self, code: &str) -> Option<TtlPolicyClaim> {
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

    fn save(&self, item: &TtlPolicyClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TtlPolicyClaim> {
        self.inner.list()
    }
}

pub fn new_cached_ttl_policy_claim_store(
    inner: Rc<dyn TtlPolicyClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TtlPolicyClaimStore> {
    Rc::new(CachedTtlPolicyClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TtlPolicyClaim>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects ttl policy claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyClaimQuery {
    pub name: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TtlPolicyClaimQuery {
    pub fn matches(&self, item: &TtlPolicyClaim) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TtlPolicyClaim]) -> Vec<TtlPolicyClaim> {
        let matched = collections::filter(items, |candidate: &TtlPolicyClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_ttl_policy_claims(store: Rc<dyn TtlPolicyClaimStore>, query: &TtlPolicyClaimQuery) -> Vec<TtlPolicyClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TtlPolicyClaimStore> = new_memory_ttl_policy_claim_store();
        let item = new_ttl_policy_claim("code-574", "name-760");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
