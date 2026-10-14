use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending nonce targets by code.
pub trait PendingNonceTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingNonceTarget>;
    fn save(&self, item: &PendingNonceTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingNonceTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingNonceTargetStore {
    pub items: RefCell<BTreeMap<String, PendingNonceTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingNonceTargetStore for MemoryPendingNonceTargetStore {
    fn find(&self, code: &str) -> Option<PendingNonceTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingNonceTarget) {
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

    fn list(&self) -> Vec<PendingNonceTarget> {
        let mut out: Vec<PendingNonceTarget> = Vec::new();
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

pub fn new_memory_pending_nonce_target_store() -> Rc<dyn PendingNonceTargetStore> {
    Rc::new(MemoryPendingNonceTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingNonceTargetStore {
    pub inner: Rc<dyn PendingNonceTargetStore>,
    pub cache: RefCell<cache::TtlCache<PendingNonceTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingNonceTargetStore for CachedPendingNonceTargetStore {
    fn find(&self, code: &str) -> Option<PendingNonceTarget> {
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

    fn save(&self, item: &PendingNonceTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingNonceTarget> {
        self.inner.list()
    }
}

pub fn new_cached_pending_nonce_target_store(
    inner: Rc<dyn PendingNonceTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingNonceTargetStore> {
    Rc::new(CachedPendingNonceTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingNonceTarget>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending nonce targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceTargetQuery {
    pub name: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingNonceTargetQuery {
    pub fn matches(&self, item: &PendingNonceTarget) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingNonceTarget]) -> Vec<PendingNonceTarget> {
        let matched = collections::filter(items, |candidate: &PendingNonceTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_nonce_targets(
    store: Rc<dyn PendingNonceTargetStore>,
    query: &PendingNonceTargetQuery,
) -> Vec<PendingNonceTarget> {
    query.apply(&store.list())
}
