use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling ttl policy agreements by code.
pub trait RollingTtlPolicyAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingTtlPolicyAgreement>;
    fn save(&self, item: &RollingTtlPolicyAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingTtlPolicyAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingTtlPolicyAgreementStore {
    pub items: RefCell<BTreeMap<String, RollingTtlPolicyAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingTtlPolicyAgreementStore for MemoryRollingTtlPolicyAgreementStore {
    fn find(&self, code: &str) -> Option<RollingTtlPolicyAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingTtlPolicyAgreement) {
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

    fn list(&self) -> Vec<RollingTtlPolicyAgreement> {
        let mut out: Vec<RollingTtlPolicyAgreement> = Vec::new();
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

pub fn new_memory_rolling_ttl_policy_agreement_store() -> Rc<dyn RollingTtlPolicyAgreementStore> {
    Rc::new(MemoryRollingTtlPolicyAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingTtlPolicyAgreementStore {
    pub inner: Rc<dyn RollingTtlPolicyAgreementStore>,
    pub cache: RefCell<cache::TtlCache<RollingTtlPolicyAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingTtlPolicyAgreementStore for CachedRollingTtlPolicyAgreementStore {
    fn find(&self, code: &str) -> Option<RollingTtlPolicyAgreement> {
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

    fn save(&self, item: &RollingTtlPolicyAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingTtlPolicyAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_ttl_policy_agreement_store(
    inner: Rc<dyn RollingTtlPolicyAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingTtlPolicyAgreementStore> {
    Rc::new(CachedRollingTtlPolicyAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingTtlPolicyAgreement>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling ttl policy agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyAgreementQuery {
    pub name: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingTtlPolicyAgreementQuery {
    pub fn matches(&self, item: &RollingTtlPolicyAgreement) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingTtlPolicyAgreement]) -> Vec<RollingTtlPolicyAgreement> {
        let matched = collections::filter(items, |candidate: &RollingTtlPolicyAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_ttl_policy_agreements(
    store: Rc<dyn RollingTtlPolicyAgreementStore>,
    query: &RollingTtlPolicyAgreementQuery,
) -> Vec<RollingTtlPolicyAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingTtlPolicyAgreementStore> = new_memory_rolling_ttl_policy_agreement_store();
        let item = new_rolling_ttl_policy_agreement("code-914", "name-791");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
