use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback overage targets by code.
pub trait FallbackOverageTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackOverageTarget>;
    fn save(&self, item: &FallbackOverageTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackOverageTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackOverageTargetStore {
    pub items: RefCell<BTreeMap<String, FallbackOverageTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackOverageTargetStore for MemoryFallbackOverageTargetStore {
    fn find(&self, code: &str) -> Option<FallbackOverageTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackOverageTarget) {
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

    fn list(&self) -> Vec<FallbackOverageTarget> {
        let mut out: Vec<FallbackOverageTarget> = Vec::new();
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

pub fn new_memory_fallback_overage_target_store() -> Rc<dyn FallbackOverageTargetStore> {
    Rc::new(MemoryFallbackOverageTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackOverageTargetStore {
    pub inner: Rc<dyn FallbackOverageTargetStore>,
    pub cache: RefCell<cache::TtlCache<FallbackOverageTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackOverageTargetStore for CachedFallbackOverageTargetStore {
    fn find(&self, code: &str) -> Option<FallbackOverageTarget> {
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

    fn save(&self, item: &FallbackOverageTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackOverageTarget> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_overage_target_store(
    inner: Rc<dyn FallbackOverageTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackOverageTargetStore> {
    Rc::new(CachedFallbackOverageTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackOverageTarget>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback overage targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageTargetQuery {
    pub region: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackOverageTargetQuery {
    pub fn matches(&self, item: &FallbackOverageTarget) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackOverageTarget]) -> Vec<FallbackOverageTarget> {
        let matched = collections::filter(items, |candidate: &FallbackOverageTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_overage_targets(
    store: Rc<dyn FallbackOverageTargetStore>,
    query: &FallbackOverageTargetQuery,
) -> Vec<FallbackOverageTarget> {
    query.apply(&store.list())
}
