use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual query agreements by code.
pub trait ManualQueryAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualQueryAgreement>;
    fn save(&self, item: &ManualQueryAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualQueryAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualQueryAgreementStore {
    pub items: RefCell<BTreeMap<String, ManualQueryAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualQueryAgreementStore for MemoryManualQueryAgreementStore {
    fn find(&self, code: &str) -> Option<ManualQueryAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualQueryAgreement) {
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

    fn list(&self) -> Vec<ManualQueryAgreement> {
        let mut out: Vec<ManualQueryAgreement> = Vec::new();
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

pub fn new_memory_manual_query_agreement_store() -> Rc<dyn ManualQueryAgreementStore> {
    Rc::new(MemoryManualQueryAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualQueryAgreementStore {
    pub inner: Rc<dyn ManualQueryAgreementStore>,
    pub cache: RefCell<cache::TtlCache<ManualQueryAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualQueryAgreementStore for CachedManualQueryAgreementStore {
    fn find(&self, code: &str) -> Option<ManualQueryAgreement> {
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

    fn save(&self, item: &ManualQueryAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualQueryAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_manual_query_agreement_store(
    inner: Rc<dyn ManualQueryAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualQueryAgreementStore> {
    Rc::new(CachedManualQueryAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualQueryAgreement>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual query agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQueryAgreementQuery {
    pub unit: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualQueryAgreementQuery {
    pub fn matches(&self, item: &ManualQueryAgreement) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[ManualQueryAgreement]) -> Vec<ManualQueryAgreement> {
        let matched = collections::filter(items, |candidate: &ManualQueryAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_query_agreements(
    store: Rc<dyn ManualQueryAgreementStore>,
    query: &ManualQueryAgreementQuery,
) -> Vec<ManualQueryAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualQueryAgreementStore> = new_memory_manual_query_agreement_store();
        let item = new_manual_query_agreement("code-607", "name-717");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
