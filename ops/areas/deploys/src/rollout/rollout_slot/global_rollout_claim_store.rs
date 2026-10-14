use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global rollout claims by code.
pub trait GlobalRolloutClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalRolloutClaim>;
    fn save(&self, item: &GlobalRolloutClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalRolloutClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalRolloutClaimStore {
    pub items: RefCell<BTreeMap<String, GlobalRolloutClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalRolloutClaimStore for MemoryGlobalRolloutClaimStore {
    fn find(&self, code: &str) -> Option<GlobalRolloutClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalRolloutClaim) {
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

    fn list(&self) -> Vec<GlobalRolloutClaim> {
        let mut out: Vec<GlobalRolloutClaim> = Vec::new();
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

pub fn new_memory_global_rollout_claim_store() -> Rc<dyn GlobalRolloutClaimStore> {
    Rc::new(MemoryGlobalRolloutClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalRolloutClaimStore {
    pub inner: Rc<dyn GlobalRolloutClaimStore>,
    pub cache: RefCell<cache::TtlCache<GlobalRolloutClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalRolloutClaimStore for CachedGlobalRolloutClaimStore {
    fn find(&self, code: &str) -> Option<GlobalRolloutClaim> {
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

    fn save(&self, item: &GlobalRolloutClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalRolloutClaim> {
        self.inner.list()
    }
}

pub fn new_cached_global_rollout_claim_store(
    inner: Rc<dyn GlobalRolloutClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalRolloutClaimStore> {
    Rc::new(CachedGlobalRolloutClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalRolloutClaim>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global rollout claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutClaimQuery {
    pub country: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalRolloutClaimQuery {
    pub fn matches(&self, item: &GlobalRolloutClaim) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalRolloutClaim]) -> Vec<GlobalRolloutClaim> {
        let matched = collections::filter(items, |candidate: &GlobalRolloutClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_rollout_claims(
    store: Rc<dyn GlobalRolloutClaimStore>,
    query: &GlobalRolloutClaimQuery,
) -> Vec<GlobalRolloutClaim> {
    query.apply(&store.list())
}
