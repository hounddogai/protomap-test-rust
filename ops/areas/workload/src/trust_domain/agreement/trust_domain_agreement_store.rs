use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps trust domain agreements by code.
pub trait TrustDomainAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TrustDomainAgreement>;
    fn save(&self, item: &TrustDomainAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TrustDomainAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTrustDomainAgreementStore {
    pub items: RefCell<BTreeMap<String, TrustDomainAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl TrustDomainAgreementStore for MemoryTrustDomainAgreementStore {
    fn find(&self, code: &str) -> Option<TrustDomainAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TrustDomainAgreement) {
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

    fn list(&self) -> Vec<TrustDomainAgreement> {
        let mut out: Vec<TrustDomainAgreement> = Vec::new();
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

pub fn new_memory_trust_domain_agreement_store() -> Rc<dyn TrustDomainAgreementStore> {
    Rc::new(MemoryTrustDomainAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTrustDomainAgreementStore {
    pub inner: Rc<dyn TrustDomainAgreementStore>,
    pub cache: RefCell<cache::TtlCache<TrustDomainAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TrustDomainAgreementStore for CachedTrustDomainAgreementStore {
    fn find(&self, code: &str) -> Option<TrustDomainAgreement> {
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

    fn save(&self, item: &TrustDomainAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TrustDomainAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_trust_domain_agreement_store(
    inner: Rc<dyn TrustDomainAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TrustDomainAgreementStore> {
    Rc::new(CachedTrustDomainAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TrustDomainAgreement>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects trust domain agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAgreementQuery {
    pub cluster: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TrustDomainAgreementQuery {
    pub fn matches(&self, item: &TrustDomainAgreement) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[TrustDomainAgreement]) -> Vec<TrustDomainAgreement> {
        let matched = collections::filter(items, |candidate: &TrustDomainAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_trust_domain_agreements(
    store: Rc<dyn TrustDomainAgreementStore>,
    query: &TrustDomainAgreementQuery,
) -> Vec<TrustDomainAgreement> {
    query.apply(&store.list())
}
