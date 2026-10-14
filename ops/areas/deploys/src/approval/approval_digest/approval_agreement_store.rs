use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval agreements by code.
pub trait ApprovalAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalAgreement>;
    fn save(&self, item: &ApprovalAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalAgreementStore {
    pub items: RefCell<BTreeMap<String, ApprovalAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalAgreementStore for MemoryApprovalAgreementStore {
    fn find(&self, code: &str) -> Option<ApprovalAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalAgreement) {
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

    fn list(&self) -> Vec<ApprovalAgreement> {
        let mut out: Vec<ApprovalAgreement> = Vec::new();
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

pub fn new_memory_approval_agreement_store() -> Rc<dyn ApprovalAgreementStore> {
    Rc::new(MemoryApprovalAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalAgreementStore {
    pub inner: Rc<dyn ApprovalAgreementStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalAgreementStore for CachedApprovalAgreementStore {
    fn find(&self, code: &str) -> Option<ApprovalAgreement> {
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

    fn save(&self, item: &ApprovalAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_approval_agreement_store(
    inner: Rc<dyn ApprovalAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalAgreementStore> {
    Rc::new(CachedApprovalAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalAgreement>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAgreementQuery {
    pub country: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalAgreementQuery {
    pub fn matches(&self, item: &ApprovalAgreement) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ApprovalAgreement]) -> Vec<ApprovalAgreement> {
        let matched = collections::filter(items, |candidate: &ApprovalAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_agreements(
    store: Rc<dyn ApprovalAgreementStore>,
    query: &ApprovalAgreementQuery,
) -> Vec<ApprovalAgreement> {
    query.apply(&store.list())
}
