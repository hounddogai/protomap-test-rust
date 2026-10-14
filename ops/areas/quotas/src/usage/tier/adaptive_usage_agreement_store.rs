use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive usage agreements by code.
pub trait AdaptiveUsageAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveUsageAgreement>;
    fn save(&self, item: &AdaptiveUsageAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveUsageAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveUsageAgreementStore {
    pub items: RefCell<BTreeMap<String, AdaptiveUsageAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveUsageAgreementStore for MemoryAdaptiveUsageAgreementStore {
    fn find(&self, code: &str) -> Option<AdaptiveUsageAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveUsageAgreement) {
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

    fn list(&self) -> Vec<AdaptiveUsageAgreement> {
        let mut out: Vec<AdaptiveUsageAgreement> = Vec::new();
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

pub fn new_memory_adaptive_usage_agreement_store() -> Rc<dyn AdaptiveUsageAgreementStore> {
    Rc::new(MemoryAdaptiveUsageAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveUsageAgreementStore {
    pub inner: Rc<dyn AdaptiveUsageAgreementStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveUsageAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveUsageAgreementStore for CachedAdaptiveUsageAgreementStore {
    fn find(&self, code: &str) -> Option<AdaptiveUsageAgreement> {
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

    fn save(&self, item: &AdaptiveUsageAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveUsageAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_usage_agreement_store(
    inner: Rc<dyn AdaptiveUsageAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveUsageAgreementStore> {
    Rc::new(CachedAdaptiveUsageAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveUsageAgreement>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive usage agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageAgreementQuery {
    pub scope: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveUsageAgreementQuery {
    pub fn matches(&self, item: &AdaptiveUsageAgreement) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveUsageAgreement]) -> Vec<AdaptiveUsageAgreement> {
        let matched = collections::filter(items, |candidate: &AdaptiveUsageAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_usage_agreements(
    store: Rc<dyn AdaptiveUsageAgreementStore>,
    query: &AdaptiveUsageAgreementQuery,
) -> Vec<AdaptiveUsageAgreement> {
    query.apply(&store.list())
}
