use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy tenant agreements by code.
pub trait LegacyTenantAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyTenantAgreement>;
    fn save(&self, item: &LegacyTenantAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyTenantAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyTenantAgreementStore {
    pub items: RefCell<BTreeMap<String, LegacyTenantAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyTenantAgreementStore for MemoryLegacyTenantAgreementStore {
    fn find(&self, code: &str) -> Option<LegacyTenantAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyTenantAgreement) {
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

    fn list(&self) -> Vec<LegacyTenantAgreement> {
        let mut out: Vec<LegacyTenantAgreement> = Vec::new();
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

pub fn new_memory_legacy_tenant_agreement_store() -> Rc<dyn LegacyTenantAgreementStore> {
    Rc::new(MemoryLegacyTenantAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyTenantAgreementStore {
    pub inner: Rc<dyn LegacyTenantAgreementStore>,
    pub cache: RefCell<cache::TtlCache<LegacyTenantAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyTenantAgreementStore for CachedLegacyTenantAgreementStore {
    fn find(&self, code: &str) -> Option<LegacyTenantAgreement> {
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

    fn save(&self, item: &LegacyTenantAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyTenantAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_tenant_agreement_store(
    inner: Rc<dyn LegacyTenantAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyTenantAgreementStore> {
    Rc::new(CachedLegacyTenantAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyTenantAgreement>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy tenant agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAgreementQuery {
    pub namespace: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyTenantAgreementQuery {
    pub fn matches(&self, item: &LegacyTenantAgreement) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyTenantAgreement]) -> Vec<LegacyTenantAgreement> {
        let matched = collections::filter(items, |candidate: &LegacyTenantAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_tenant_agreements(
    store: Rc<dyn LegacyTenantAgreementStore>,
    query: &LegacyTenantAgreementQuery,
) -> Vec<LegacyTenantAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyTenantAgreementStore> = new_memory_legacy_tenant_agreement_store();
        let item = new_legacy_tenant_agreement("code-567", "title-703");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
