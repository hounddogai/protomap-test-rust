use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback rollback tiers by code.
pub trait FallbackRollbackTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackRollbackTier>;
    fn save(&self, item: &FallbackRollbackTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackRollbackTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackRollbackTierStore {
    pub items: RefCell<BTreeMap<String, FallbackRollbackTier>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackRollbackTierStore for MemoryFallbackRollbackTierStore {
    fn find(&self, code: &str) -> Option<FallbackRollbackTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackRollbackTier) {
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

    fn list(&self) -> Vec<FallbackRollbackTier> {
        let mut out: Vec<FallbackRollbackTier> = Vec::new();
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

pub fn new_memory_fallback_rollback_tier_store() -> Rc<dyn FallbackRollbackTierStore> {
    Rc::new(MemoryFallbackRollbackTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackRollbackTierStore {
    pub inner: Rc<dyn FallbackRollbackTierStore>,
    pub cache: RefCell<cache::TtlCache<FallbackRollbackTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackRollbackTierStore for CachedFallbackRollbackTierStore {
    fn find(&self, code: &str) -> Option<FallbackRollbackTier> {
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

    fn save(&self, item: &FallbackRollbackTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackRollbackTier> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_rollback_tier_store(
    inner: Rc<dyn FallbackRollbackTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackRollbackTierStore> {
    Rc::new(CachedFallbackRollbackTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackRollbackTier>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback rollback tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackTierQuery {
    pub segment: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackRollbackTierQuery {
    pub fn matches(&self, item: &FallbackRollbackTier) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackRollbackTier]) -> Vec<FallbackRollbackTier> {
        let matched = collections::filter(items, |candidate: &FallbackRollbackTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_rollback_tiers(
    store: Rc<dyn FallbackRollbackTierStore>,
    query: &FallbackRollbackTierQuery,
) -> Vec<FallbackRollbackTier> {
    query.apply(&store.list())
}
