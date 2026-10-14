use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy trust domain estimates by code.
pub trait LegacyTrustDomainEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyTrustDomainEstimate>;
    fn save(&self, item: &LegacyTrustDomainEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyTrustDomainEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyTrustDomainEstimateStore {
    pub items: RefCell<BTreeMap<String, LegacyTrustDomainEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyTrustDomainEstimateStore for MemoryLegacyTrustDomainEstimateStore {
    fn find(&self, code: &str) -> Option<LegacyTrustDomainEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyTrustDomainEstimate) {
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

    fn list(&self) -> Vec<LegacyTrustDomainEstimate> {
        let mut out: Vec<LegacyTrustDomainEstimate> = Vec::new();
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

pub fn new_memory_legacy_trust_domain_estimate_store() -> Rc<dyn LegacyTrustDomainEstimateStore> {
    Rc::new(MemoryLegacyTrustDomainEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyTrustDomainEstimateStore {
    pub inner: Rc<dyn LegacyTrustDomainEstimateStore>,
    pub cache: RefCell<cache::TtlCache<LegacyTrustDomainEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyTrustDomainEstimateStore for CachedLegacyTrustDomainEstimateStore {
    fn find(&self, code: &str) -> Option<LegacyTrustDomainEstimate> {
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

    fn save(&self, item: &LegacyTrustDomainEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyTrustDomainEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_trust_domain_estimate_store(
    inner: Rc<dyn LegacyTrustDomainEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyTrustDomainEstimateStore> {
    Rc::new(CachedLegacyTrustDomainEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyTrustDomainEstimate>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy trust domain estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainEstimateQuery {
    pub unit: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyTrustDomainEstimateQuery {
    pub fn matches(&self, item: &LegacyTrustDomainEstimate) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyTrustDomainEstimate]) -> Vec<LegacyTrustDomainEstimate> {
        let matched = collections::filter(items, |candidate: &LegacyTrustDomainEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_trust_domain_estimates(
    store: Rc<dyn LegacyTrustDomainEstimateStore>,
    query: &LegacyTrustDomainEstimateQuery,
) -> Vec<LegacyTrustDomainEstimate> {
    query.apply(&store.list())
}
