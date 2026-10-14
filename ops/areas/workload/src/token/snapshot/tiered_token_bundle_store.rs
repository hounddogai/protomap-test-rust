use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered token bundles by code.
pub trait TieredTokenBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredTokenBundle>;
    fn save(&self, item: &TieredTokenBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredTokenBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredTokenBundleStore {
    pub items: RefCell<BTreeMap<String, TieredTokenBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredTokenBundleStore for MemoryTieredTokenBundleStore {
    fn find(&self, code: &str) -> Option<TieredTokenBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredTokenBundle) {
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

    fn list(&self) -> Vec<TieredTokenBundle> {
        let mut out: Vec<TieredTokenBundle> = Vec::new();
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

pub fn new_memory_tiered_token_bundle_store() -> Rc<dyn TieredTokenBundleStore> {
    Rc::new(MemoryTieredTokenBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredTokenBundleStore {
    pub inner: Rc<dyn TieredTokenBundleStore>,
    pub cache: RefCell<cache::TtlCache<TieredTokenBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredTokenBundleStore for CachedTieredTokenBundleStore {
    fn find(&self, code: &str) -> Option<TieredTokenBundle> {
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

    fn save(&self, item: &TieredTokenBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredTokenBundle> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_token_bundle_store(
    inner: Rc<dyn TieredTokenBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredTokenBundleStore> {
    Rc::new(CachedTieredTokenBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredTokenBundle>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered token bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenBundleQuery {
    pub name: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredTokenBundleQuery {
    pub fn matches(&self, item: &TieredTokenBundle) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredTokenBundle]) -> Vec<TieredTokenBundle> {
        let matched = collections::filter(items, |candidate: &TieredTokenBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_token_bundles(
    store: Rc<dyn TieredTokenBundleStore>,
    query: &TieredTokenBundleQuery,
) -> Vec<TieredTokenBundle> {
    query.apply(&store.list())
}
