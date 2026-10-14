use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tenant agreements by code.
pub trait TenantAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TenantAgreement>;
    fn save(&self, item: &TenantAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TenantAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTenantAgreementStore {
    pub items: RefCell<BTreeMap<String, TenantAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl TenantAgreementStore for MemoryTenantAgreementStore {
    fn find(&self, code: &str) -> Option<TenantAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TenantAgreement) {
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

    fn list(&self) -> Vec<TenantAgreement> {
        let mut out: Vec<TenantAgreement> = Vec::new();
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

pub fn new_memory_tenant_agreement_store() -> Rc<dyn TenantAgreementStore> {
    Rc::new(MemoryTenantAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTenantAgreementStore {
    pub inner: Rc<dyn TenantAgreementStore>,
    pub cache: RefCell<cache::TtlCache<TenantAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TenantAgreementStore for CachedTenantAgreementStore {
    fn find(&self, code: &str) -> Option<TenantAgreement> {
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

    fn save(&self, item: &TenantAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TenantAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_tenant_agreement_store(
    inner: Rc<dyn TenantAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TenantAgreementStore> {
    Rc::new(CachedTenantAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TenantAgreement>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tenant agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAgreementQuery {
    pub scope: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TenantAgreementQuery {
    pub fn matches(&self, item: &TenantAgreement) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TenantAgreement]) -> Vec<TenantAgreement> {
        let matched = collections::filter(items, |candidate: &TenantAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tenant_agreements(
    store: Rc<dyn TenantAgreementStore>,
    query: &TenantAgreementQuery,
) -> Vec<TenantAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TenantAgreementStore> = new_memory_tenant_agreement_store();
        let item = new_tenant_agreement("code-138", "title-402");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
