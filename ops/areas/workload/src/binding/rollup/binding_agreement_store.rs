use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding agreements by code.
pub trait BindingAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingAgreement>;
    fn save(&self, item: &BindingAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingAgreementStore {
    pub items: RefCell<BTreeMap<String, BindingAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingAgreementStore for MemoryBindingAgreementStore {
    fn find(&self, code: &str) -> Option<BindingAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingAgreement) {
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

    fn list(&self) -> Vec<BindingAgreement> {
        let mut out: Vec<BindingAgreement> = Vec::new();
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

pub fn new_memory_binding_agreement_store() -> Rc<dyn BindingAgreementStore> {
    Rc::new(MemoryBindingAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingAgreementStore {
    pub inner: Rc<dyn BindingAgreementStore>,
    pub cache: RefCell<cache::TtlCache<BindingAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingAgreementStore for CachedBindingAgreementStore {
    fn find(&self, code: &str) -> Option<BindingAgreement> {
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

    fn save(&self, item: &BindingAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_binding_agreement_store(
    inner: Rc<dyn BindingAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingAgreementStore> {
    Rc::new(CachedBindingAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingAgreement>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAgreementQuery {
    pub namespace: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingAgreementQuery {
    pub fn matches(&self, item: &BindingAgreement) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingAgreement]) -> Vec<BindingAgreement> {
        let matched = collections::filter(items, |candidate: &BindingAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_agreements(
    store: Rc<dyn BindingAgreementStore>,
    query: &BindingAgreementQuery,
) -> Vec<BindingAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingAgreementStore> = new_memory_binding_agreement_store();
        let item = new_binding_agreement("code-98", "name-865");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
