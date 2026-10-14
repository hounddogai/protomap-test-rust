use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient federation agreements by code.
pub trait LenientFederationAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientFederationAgreement>;
    fn save(&self, item: &LenientFederationAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientFederationAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientFederationAgreementStore {
    pub items: RefCell<BTreeMap<String, LenientFederationAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientFederationAgreementStore for MemoryLenientFederationAgreementStore {
    fn find(&self, code: &str) -> Option<LenientFederationAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientFederationAgreement) {
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

    fn list(&self) -> Vec<LenientFederationAgreement> {
        let mut out: Vec<LenientFederationAgreement> = Vec::new();
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

pub fn new_memory_lenient_federation_agreement_store() -> Rc<dyn LenientFederationAgreementStore> {
    Rc::new(MemoryLenientFederationAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientFederationAgreementStore {
    pub inner: Rc<dyn LenientFederationAgreementStore>,
    pub cache: RefCell<cache::TtlCache<LenientFederationAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientFederationAgreementStore for CachedLenientFederationAgreementStore {
    fn find(&self, code: &str) -> Option<LenientFederationAgreement> {
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

    fn save(&self, item: &LenientFederationAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientFederationAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_federation_agreement_store(
    inner: Rc<dyn LenientFederationAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientFederationAgreementStore> {
    Rc::new(CachedLenientFederationAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientFederationAgreement>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient federation agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationAgreementQuery {
    pub source: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientFederationAgreementQuery {
    pub fn matches(&self, item: &LenientFederationAgreement) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientFederationAgreement]) -> Vec<LenientFederationAgreement> {
        let matched = collections::filter(items, |candidate: &LenientFederationAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_federation_agreements(
    store: Rc<dyn LenientFederationAgreementStore>,
    query: &LenientFederationAgreementQuery,
) -> Vec<LenientFederationAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientFederationAgreementStore> = new_memory_lenient_federation_agreement_store();
        let item = new_lenient_federation_agreement("code-993", "title-118");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
