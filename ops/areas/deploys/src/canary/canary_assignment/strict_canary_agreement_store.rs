use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict canary agreements by code.
pub trait StrictCanaryAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictCanaryAgreement>;
    fn save(&self, item: &StrictCanaryAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictCanaryAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictCanaryAgreementStore {
    pub items: RefCell<BTreeMap<String, StrictCanaryAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictCanaryAgreementStore for MemoryStrictCanaryAgreementStore {
    fn find(&self, code: &str) -> Option<StrictCanaryAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictCanaryAgreement) {
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

    fn list(&self) -> Vec<StrictCanaryAgreement> {
        let mut out: Vec<StrictCanaryAgreement> = Vec::new();
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

pub fn new_memory_strict_canary_agreement_store() -> Rc<dyn StrictCanaryAgreementStore> {
    Rc::new(MemoryStrictCanaryAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictCanaryAgreementStore {
    pub inner: Rc<dyn StrictCanaryAgreementStore>,
    pub cache: RefCell<cache::TtlCache<StrictCanaryAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictCanaryAgreementStore for CachedStrictCanaryAgreementStore {
    fn find(&self, code: &str) -> Option<StrictCanaryAgreement> {
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

    fn save(&self, item: &StrictCanaryAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictCanaryAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_strict_canary_agreement_store(
    inner: Rc<dyn StrictCanaryAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictCanaryAgreementStore> {
    Rc::new(CachedStrictCanaryAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictCanaryAgreement>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict canary agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryAgreementQuery {
    pub category: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictCanaryAgreementQuery {
    pub fn matches(&self, item: &StrictCanaryAgreement) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictCanaryAgreement]) -> Vec<StrictCanaryAgreement> {
        let matched = collections::filter(items, |candidate: &StrictCanaryAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_canary_agreements(
    store: Rc<dyn StrictCanaryAgreementStore>,
    query: &StrictCanaryAgreementQuery,
) -> Vec<StrictCanaryAgreement> {
    query.apply(&store.list())
}
