use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps ttl policy agreements by code.
pub trait TtlPolicyAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TtlPolicyAgreement>;
    fn save(&self, item: &TtlPolicyAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TtlPolicyAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTtlPolicyAgreementStore {
    pub items: RefCell<BTreeMap<String, TtlPolicyAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl TtlPolicyAgreementStore for MemoryTtlPolicyAgreementStore {
    fn find(&self, code: &str) -> Option<TtlPolicyAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TtlPolicyAgreement) {
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

    fn list(&self) -> Vec<TtlPolicyAgreement> {
        let mut out: Vec<TtlPolicyAgreement> = Vec::new();
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

pub fn new_memory_ttl_policy_agreement_store() -> Rc<dyn TtlPolicyAgreementStore> {
    Rc::new(MemoryTtlPolicyAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTtlPolicyAgreementStore {
    pub inner: Rc<dyn TtlPolicyAgreementStore>,
    pub cache: RefCell<cache::TtlCache<TtlPolicyAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TtlPolicyAgreementStore for CachedTtlPolicyAgreementStore {
    fn find(&self, code: &str) -> Option<TtlPolicyAgreement> {
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

    fn save(&self, item: &TtlPolicyAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TtlPolicyAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_ttl_policy_agreement_store(
    inner: Rc<dyn TtlPolicyAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TtlPolicyAgreementStore> {
    Rc::new(CachedTtlPolicyAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TtlPolicyAgreement>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects ttl policy agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAgreementQuery {
    pub name: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TtlPolicyAgreementQuery {
    pub fn matches(&self, item: &TtlPolicyAgreement) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TtlPolicyAgreement]) -> Vec<TtlPolicyAgreement> {
        let matched = collections::filter(items, |candidate: &TtlPolicyAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_ttl_policy_agreements(
    store: Rc<dyn TtlPolicyAgreementStore>,
    query: &TtlPolicyAgreementQuery,
) -> Vec<TtlPolicyAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TtlPolicyAgreementStore> = new_memory_ttl_policy_agreement_store();
        let item = new_ttl_policy_agreement("code-606", "name-247");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
