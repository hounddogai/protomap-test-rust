use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback egress policies by code.
pub trait FallbackEgressPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackEgressPolicy>;
    fn save(&self, item: &FallbackEgressPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackEgressPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackEgressPolicyStore {
    pub items: RefCell<BTreeMap<String, FallbackEgressPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackEgressPolicyStore for MemoryFallbackEgressPolicyStore {
    fn find(&self, code: &str) -> Option<FallbackEgressPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackEgressPolicy) {
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

    fn list(&self) -> Vec<FallbackEgressPolicy> {
        let mut out: Vec<FallbackEgressPolicy> = Vec::new();
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

pub fn new_memory_fallback_egress_policy_store() -> Rc<dyn FallbackEgressPolicyStore> {
    Rc::new(MemoryFallbackEgressPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackEgressPolicyStore {
    pub inner: Rc<dyn FallbackEgressPolicyStore>,
    pub cache: RefCell<cache::TtlCache<FallbackEgressPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackEgressPolicyStore for CachedFallbackEgressPolicyStore {
    fn find(&self, code: &str) -> Option<FallbackEgressPolicy> {
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

    fn save(&self, item: &FallbackEgressPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackEgressPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_egress_policy_store(
    inner: Rc<dyn FallbackEgressPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackEgressPolicyStore> {
    Rc::new(CachedFallbackEgressPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackEgressPolicy>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback egress policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPolicyQuery {
    pub locale: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackEgressPolicyQuery {
    pub fn matches(&self, item: &FallbackEgressPolicy) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
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

    pub fn apply(&self, items: &[FallbackEgressPolicy]) -> Vec<FallbackEgressPolicy> {
        let matched = collections::filter(items, |candidate: &FallbackEgressPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_egress_policies(
    store: Rc<dyn FallbackEgressPolicyStore>,
    query: &FallbackEgressPolicyQuery,
) -> Vec<FallbackEgressPolicy> {
    query.apply(&store.list())
}
