use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps federation policies by code.
pub trait FederationPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FederationPolicy>;
    fn save(&self, item: &FederationPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FederationPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFederationPolicyStore {
    pub items: RefCell<BTreeMap<String, FederationPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl FederationPolicyStore for MemoryFederationPolicyStore {
    fn find(&self, code: &str) -> Option<FederationPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FederationPolicy) {
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

    fn list(&self) -> Vec<FederationPolicy> {
        let mut out: Vec<FederationPolicy> = Vec::new();
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

pub fn new_memory_federation_policy_store() -> Rc<dyn FederationPolicyStore> {
    Rc::new(MemoryFederationPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFederationPolicyStore {
    pub inner: Rc<dyn FederationPolicyStore>,
    pub cache: RefCell<cache::TtlCache<FederationPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FederationPolicyStore for CachedFederationPolicyStore {
    fn find(&self, code: &str) -> Option<FederationPolicy> {
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

    fn save(&self, item: &FederationPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FederationPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_federation_policy_store(
    inner: Rc<dyn FederationPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FederationPolicyStore> {
    Rc::new(CachedFederationPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FederationPolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects federation policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationPolicyQuery {
    pub region: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FederationPolicyQuery {
    pub fn matches(&self, item: &FederationPolicy) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FederationPolicy]) -> Vec<FederationPolicy> {
        let matched = collections::filter(items, |candidate: &FederationPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_federation_policies(
    store: Rc<dyn FederationPolicyStore>,
    query: &FederationPolicyQuery,
) -> Vec<FederationPolicy> {
    query.apply(&store.list())
}
