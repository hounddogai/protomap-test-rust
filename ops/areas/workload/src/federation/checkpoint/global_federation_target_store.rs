use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global federation targets by code.
pub trait GlobalFederationTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalFederationTarget>;
    fn save(&self, item: &GlobalFederationTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalFederationTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalFederationTargetStore {
    pub items: RefCell<BTreeMap<String, GlobalFederationTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalFederationTargetStore for MemoryGlobalFederationTargetStore {
    fn find(&self, code: &str) -> Option<GlobalFederationTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalFederationTarget) {
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

    fn list(&self) -> Vec<GlobalFederationTarget> {
        let mut out: Vec<GlobalFederationTarget> = Vec::new();
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

pub fn new_memory_global_federation_target_store() -> Rc<dyn GlobalFederationTargetStore> {
    Rc::new(MemoryGlobalFederationTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalFederationTargetStore {
    pub inner: Rc<dyn GlobalFederationTargetStore>,
    pub cache: RefCell<cache::TtlCache<GlobalFederationTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalFederationTargetStore for CachedGlobalFederationTargetStore {
    fn find(&self, code: &str) -> Option<GlobalFederationTarget> {
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

    fn save(&self, item: &GlobalFederationTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalFederationTarget> {
        self.inner.list()
    }
}

pub fn new_cached_global_federation_target_store(
    inner: Rc<dyn GlobalFederationTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalFederationTargetStore> {
    Rc::new(CachedGlobalFederationTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalFederationTarget>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global federation targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationTargetQuery {
    pub unit: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalFederationTargetQuery {
    pub fn matches(&self, item: &GlobalFederationTarget) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalFederationTarget]) -> Vec<GlobalFederationTarget> {
        let matched = collections::filter(items, |candidate: &GlobalFederationTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_federation_targets(
    store: Rc<dyn GlobalFederationTargetStore>,
    query: &GlobalFederationTargetQuery,
) -> Vec<GlobalFederationTarget> {
    query.apply(&store.list())
}
