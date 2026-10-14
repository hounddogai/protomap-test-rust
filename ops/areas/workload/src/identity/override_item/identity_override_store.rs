use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity overrides by code.
pub trait IdentityOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityOverride>;
    fn save(&self, item: &IdentityOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityOverrideStore {
    pub items: RefCell<BTreeMap<String, IdentityOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityOverrideStore for MemoryIdentityOverrideStore {
    fn find(&self, code: &str) -> Option<IdentityOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityOverride) {
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

    fn list(&self) -> Vec<IdentityOverride> {
        let mut out: Vec<IdentityOverride> = Vec::new();
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

pub fn new_memory_identity_override_store() -> Rc<dyn IdentityOverrideStore> {
    Rc::new(MemoryIdentityOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityOverrideStore {
    pub inner: Rc<dyn IdentityOverrideStore>,
    pub cache: RefCell<cache::TtlCache<IdentityOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityOverrideStore for CachedIdentityOverrideStore {
    fn find(&self, code: &str) -> Option<IdentityOverride> {
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

    fn save(&self, item: &IdentityOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityOverride> {
        self.inner.list()
    }
}

pub fn new_cached_identity_override_store(
    inner: Rc<dyn IdentityOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityOverrideStore> {
    Rc::new(CachedIdentityOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityOverride>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityOverrideQuery {
    pub name: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityOverrideQuery {
    pub fn matches(&self, item: &IdentityOverride) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IdentityOverride]) -> Vec<IdentityOverride> {
        let matched = collections::filter(items, |candidate: &IdentityOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_overrides(
    store: Rc<dyn IdentityOverrideStore>,
    query: &IdentityOverrideQuery,
) -> Vec<IdentityOverride> {
    query.apply(&store.list())
}
