use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding claims by code.
pub trait BindingClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingClaim>;
    fn save(&self, item: &BindingClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingClaimStore {
    pub items: RefCell<BTreeMap<String, BindingClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingClaimStore for MemoryBindingClaimStore {
    fn find(&self, code: &str) -> Option<BindingClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingClaim) {
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

    fn list(&self) -> Vec<BindingClaim> {
        let mut out: Vec<BindingClaim> = Vec::new();
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

pub fn new_memory_binding_claim_store() -> Rc<dyn BindingClaimStore> {
    Rc::new(MemoryBindingClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingClaimStore {
    pub inner: Rc<dyn BindingClaimStore>,
    pub cache: RefCell<cache::TtlCache<BindingClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingClaimStore for CachedBindingClaimStore {
    fn find(&self, code: &str) -> Option<BindingClaim> {
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

    fn save(&self, item: &BindingClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingClaim> {
        self.inner.list()
    }
}

pub fn new_cached_binding_claim_store(
    inner: Rc<dyn BindingClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingClaimStore> {
    Rc::new(CachedBindingClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingClaim>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingClaimQuery {
    pub segment: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingClaimQuery {
    pub fn matches(&self, item: &BindingClaim) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingClaim]) -> Vec<BindingClaim> {
        let matched = collections::filter(items, |candidate: &BindingClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_claims(store: Rc<dyn BindingClaimStore>, query: &BindingClaimQuery) -> Vec<BindingClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingClaimStore> = new_memory_binding_claim_store();
        let item = new_binding_claim("code-272", "name-363");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
