use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps federation windows by code.
pub trait FederationWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FederationWindow>;
    fn save(&self, item: &FederationWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FederationWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFederationWindowStore {
    pub items: RefCell<BTreeMap<String, FederationWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl FederationWindowStore for MemoryFederationWindowStore {
    fn find(&self, code: &str) -> Option<FederationWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FederationWindow) {
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

    fn list(&self) -> Vec<FederationWindow> {
        let mut out: Vec<FederationWindow> = Vec::new();
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

pub fn new_memory_federation_window_store() -> Rc<dyn FederationWindowStore> {
    Rc::new(MemoryFederationWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFederationWindowStore {
    pub inner: Rc<dyn FederationWindowStore>,
    pub cache: RefCell<cache::TtlCache<FederationWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FederationWindowStore for CachedFederationWindowStore {
    fn find(&self, code: &str) -> Option<FederationWindow> {
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

    fn save(&self, item: &FederationWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FederationWindow> {
        self.inner.list()
    }
}

pub fn new_cached_federation_window_store(
    inner: Rc<dyn FederationWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FederationWindowStore> {
    Rc::new(CachedFederationWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FederationWindow>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects federation windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationWindowQuery {
    pub cluster: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FederationWindowQuery {
    pub fn matches(&self, item: &FederationWindow) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[FederationWindow]) -> Vec<FederationWindow> {
        let matched = collections::filter(items, |candidate: &FederationWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_federation_windows(
    store: Rc<dyn FederationWindowStore>,
    query: &FederationWindowQuery,
) -> Vec<FederationWindow> {
    query.apply(&store.list())
}
