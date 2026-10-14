use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered egress overrides by code.
pub trait TieredEgressOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredEgressOverride>;
    fn save(&self, item: &TieredEgressOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredEgressOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredEgressOverrideStore {
    pub items: RefCell<BTreeMap<String, TieredEgressOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredEgressOverrideStore for MemoryTieredEgressOverrideStore {
    fn find(&self, code: &str) -> Option<TieredEgressOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredEgressOverride) {
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

    fn list(&self) -> Vec<TieredEgressOverride> {
        let mut out: Vec<TieredEgressOverride> = Vec::new();
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

pub fn new_memory_tiered_egress_override_store() -> Rc<dyn TieredEgressOverrideStore> {
    Rc::new(MemoryTieredEgressOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredEgressOverrideStore {
    pub inner: Rc<dyn TieredEgressOverrideStore>,
    pub cache: RefCell<cache::TtlCache<TieredEgressOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredEgressOverrideStore for CachedTieredEgressOverrideStore {
    fn find(&self, code: &str) -> Option<TieredEgressOverride> {
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

    fn save(&self, item: &TieredEgressOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredEgressOverride> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_egress_override_store(
    inner: Rc<dyn TieredEgressOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredEgressOverrideStore> {
    Rc::new(CachedTieredEgressOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredEgressOverride>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered egress overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressOverrideQuery {
    pub title: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredEgressOverrideQuery {
    pub fn matches(&self, item: &TieredEgressOverride) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredEgressOverride]) -> Vec<TieredEgressOverride> {
        let matched = collections::filter(items, |candidate: &TieredEgressOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_egress_overrides(
    store: Rc<dyn TieredEgressOverrideStore>,
    query: &TieredEgressOverrideQuery,
) -> Vec<TieredEgressOverride> {
    query.apply(&store.list())
}
