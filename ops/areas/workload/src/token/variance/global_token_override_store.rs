use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global token overrides by code.
pub trait GlobalTokenOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalTokenOverride>;
    fn save(&self, item: &GlobalTokenOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalTokenOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalTokenOverrideStore {
    pub items: RefCell<BTreeMap<String, GlobalTokenOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalTokenOverrideStore for MemoryGlobalTokenOverrideStore {
    fn find(&self, code: &str) -> Option<GlobalTokenOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalTokenOverride) {
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

    fn list(&self) -> Vec<GlobalTokenOverride> {
        let mut out: Vec<GlobalTokenOverride> = Vec::new();
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

pub fn new_memory_global_token_override_store() -> Rc<dyn GlobalTokenOverrideStore> {
    Rc::new(MemoryGlobalTokenOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalTokenOverrideStore {
    pub inner: Rc<dyn GlobalTokenOverrideStore>,
    pub cache: RefCell<cache::TtlCache<GlobalTokenOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalTokenOverrideStore for CachedGlobalTokenOverrideStore {
    fn find(&self, code: &str) -> Option<GlobalTokenOverride> {
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

    fn save(&self, item: &GlobalTokenOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalTokenOverride> {
        self.inner.list()
    }
}

pub fn new_cached_global_token_override_store(
    inner: Rc<dyn GlobalTokenOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalTokenOverrideStore> {
    Rc::new(CachedGlobalTokenOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalTokenOverride>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global token overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenOverrideQuery {
    pub cluster: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalTokenOverrideQuery {
    pub fn matches(&self, item: &GlobalTokenOverride) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalTokenOverride]) -> Vec<GlobalTokenOverride> {
        let matched = collections::filter(items, |candidate: &GlobalTokenOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_token_overrides(
    store: Rc<dyn GlobalTokenOverrideStore>,
    query: &GlobalTokenOverrideQuery,
) -> Vec<GlobalTokenOverride> {
    query.apply(&store.list())
}
