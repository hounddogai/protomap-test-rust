use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional failover agreements by code.
pub trait RegionalFailoverAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalFailoverAgreement>;
    fn save(&self, item: &RegionalFailoverAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalFailoverAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalFailoverAgreementStore {
    pub items: RefCell<BTreeMap<String, RegionalFailoverAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalFailoverAgreementStore for MemoryRegionalFailoverAgreementStore {
    fn find(&self, code: &str) -> Option<RegionalFailoverAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalFailoverAgreement) {
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

    fn list(&self) -> Vec<RegionalFailoverAgreement> {
        let mut out: Vec<RegionalFailoverAgreement> = Vec::new();
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

pub fn new_memory_regional_failover_agreement_store() -> Rc<dyn RegionalFailoverAgreementStore> {
    Rc::new(MemoryRegionalFailoverAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalFailoverAgreementStore {
    pub inner: Rc<dyn RegionalFailoverAgreementStore>,
    pub cache: RefCell<cache::TtlCache<RegionalFailoverAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalFailoverAgreementStore for CachedRegionalFailoverAgreementStore {
    fn find(&self, code: &str) -> Option<RegionalFailoverAgreement> {
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

    fn save(&self, item: &RegionalFailoverAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalFailoverAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_regional_failover_agreement_store(
    inner: Rc<dyn RegionalFailoverAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalFailoverAgreementStore> {
    Rc::new(CachedRegionalFailoverAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalFailoverAgreement>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional failover agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverAgreementQuery {
    pub locale: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalFailoverAgreementQuery {
    pub fn matches(&self, item: &RegionalFailoverAgreement) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalFailoverAgreement]) -> Vec<RegionalFailoverAgreement> {
        let matched = collections::filter(items, |candidate: &RegionalFailoverAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_failover_agreements(
    store: Rc<dyn RegionalFailoverAgreementStore>,
    query: &RegionalFailoverAgreementQuery,
) -> Vec<RegionalFailoverAgreement> {
    query.apply(&store.list())
}
