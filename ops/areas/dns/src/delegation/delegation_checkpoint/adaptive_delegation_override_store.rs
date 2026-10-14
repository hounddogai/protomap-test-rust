use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive delegation overrides by code.
pub trait AdaptiveDelegationOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveDelegationOverride>;
    fn save(&self, item: &AdaptiveDelegationOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveDelegationOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveDelegationOverrideStore {
    pub items: RefCell<BTreeMap<String, AdaptiveDelegationOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveDelegationOverrideStore for MemoryAdaptiveDelegationOverrideStore {
    fn find(&self, code: &str) -> Option<AdaptiveDelegationOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveDelegationOverride) {
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

    fn list(&self) -> Vec<AdaptiveDelegationOverride> {
        let mut out: Vec<AdaptiveDelegationOverride> = Vec::new();
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

pub fn new_memory_adaptive_delegation_override_store() -> Rc<dyn AdaptiveDelegationOverrideStore> {
    Rc::new(MemoryAdaptiveDelegationOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveDelegationOverrideStore {
    pub inner: Rc<dyn AdaptiveDelegationOverrideStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveDelegationOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveDelegationOverrideStore for CachedAdaptiveDelegationOverrideStore {
    fn find(&self, code: &str) -> Option<AdaptiveDelegationOverride> {
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

    fn save(&self, item: &AdaptiveDelegationOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveDelegationOverride> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_delegation_override_store(
    inner: Rc<dyn AdaptiveDelegationOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveDelegationOverrideStore> {
    Rc::new(CachedAdaptiveDelegationOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveDelegationOverride>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive delegation overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationOverrideQuery {
    pub channel: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveDelegationOverrideQuery {
    pub fn matches(&self, item: &AdaptiveDelegationOverride) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveDelegationOverride]) -> Vec<AdaptiveDelegationOverride> {
        let matched = collections::filter(items, |candidate: &AdaptiveDelegationOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_delegation_overrides(
    store: Rc<dyn AdaptiveDelegationOverrideStore>,
    query: &AdaptiveDelegationOverrideQuery,
) -> Vec<AdaptiveDelegationOverride> {
    query.apply(&store.list())
}
