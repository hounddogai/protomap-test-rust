use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy runbook agreements by code.
pub trait LegacyRunbookAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyRunbookAgreement>;
    fn save(&self, item: &LegacyRunbookAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyRunbookAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyRunbookAgreementStore {
    pub items: RefCell<BTreeMap<String, LegacyRunbookAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyRunbookAgreementStore for MemoryLegacyRunbookAgreementStore {
    fn find(&self, code: &str) -> Option<LegacyRunbookAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyRunbookAgreement) {
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

    fn list(&self) -> Vec<LegacyRunbookAgreement> {
        let mut out: Vec<LegacyRunbookAgreement> = Vec::new();
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

pub fn new_memory_legacy_runbook_agreement_store() -> Rc<dyn LegacyRunbookAgreementStore> {
    Rc::new(MemoryLegacyRunbookAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyRunbookAgreementStore {
    pub inner: Rc<dyn LegacyRunbookAgreementStore>,
    pub cache: RefCell<cache::TtlCache<LegacyRunbookAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyRunbookAgreementStore for CachedLegacyRunbookAgreementStore {
    fn find(&self, code: &str) -> Option<LegacyRunbookAgreement> {
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

    fn save(&self, item: &LegacyRunbookAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyRunbookAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_runbook_agreement_store(
    inner: Rc<dyn LegacyRunbookAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyRunbookAgreementStore> {
    Rc::new(CachedLegacyRunbookAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyRunbookAgreement>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy runbook agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAgreementQuery {
    pub namespace: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyRunbookAgreementQuery {
    pub fn matches(&self, item: &LegacyRunbookAgreement) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyRunbookAgreement]) -> Vec<LegacyRunbookAgreement> {
        let matched = collections::filter(items, |candidate: &LegacyRunbookAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_runbook_agreements(
    store: Rc<dyn LegacyRunbookAgreementStore>,
    query: &LegacyRunbookAgreementQuery,
) -> Vec<LegacyRunbookAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyRunbookAgreementStore> = new_memory_legacy_runbook_agreement_store();
        let item = new_legacy_runbook_agreement("code-459", "name-798");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
