use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps delegation tiers by code.
pub trait DelegationTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DelegationTier>;
    fn save(&self, item: &DelegationTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DelegationTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelegationTierStore {
    pub items: RefCell<BTreeMap<String, DelegationTier>>,
    pub order: RefCell<Vec<String>>,
}

impl DelegationTierStore for MemoryDelegationTierStore {
    fn find(&self, code: &str) -> Option<DelegationTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DelegationTier) {
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

    fn list(&self) -> Vec<DelegationTier> {
        let mut out: Vec<DelegationTier> = Vec::new();
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

pub fn new_memory_delegation_tier_store() -> Rc<dyn DelegationTierStore> {
    Rc::new(MemoryDelegationTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDelegationTierStore {
    pub inner: Rc<dyn DelegationTierStore>,
    pub cache: RefCell<cache::TtlCache<DelegationTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DelegationTierStore for CachedDelegationTierStore {
    fn find(&self, code: &str) -> Option<DelegationTier> {
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

    fn save(&self, item: &DelegationTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DelegationTier> {
        self.inner.list()
    }
}

pub fn new_cached_delegation_tier_store(
    inner: Rc<dyn DelegationTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DelegationTierStore> {
    Rc::new(CachedDelegationTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DelegationTier>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects delegation tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTierQuery {
    pub category: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DelegationTierQuery {
    pub fn matches(&self, item: &DelegationTier) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DelegationTier]) -> Vec<DelegationTier> {
        let matched = collections::filter(items, |candidate: &DelegationTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_delegation_tiers(store: Rc<dyn DelegationTierStore>, query: &DelegationTierQuery) -> Vec<DelegationTier> {
    query.apply(&store.list())
}
