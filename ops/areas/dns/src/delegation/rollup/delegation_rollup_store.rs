use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps delegation rollups by code.
pub trait DelegationRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DelegationRollup>;
    fn save(&self, item: &DelegationRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DelegationRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelegationRollupStore {
    pub items: RefCell<BTreeMap<String, DelegationRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl DelegationRollupStore for MemoryDelegationRollupStore {
    fn find(&self, code: &str) -> Option<DelegationRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DelegationRollup) {
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

    fn list(&self) -> Vec<DelegationRollup> {
        let mut out: Vec<DelegationRollup> = Vec::new();
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

pub fn new_memory_delegation_rollup_store() -> Rc<dyn DelegationRollupStore> {
    Rc::new(MemoryDelegationRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDelegationRollupStore {
    pub inner: Rc<dyn DelegationRollupStore>,
    pub cache: RefCell<cache::TtlCache<DelegationRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DelegationRollupStore for CachedDelegationRollupStore {
    fn find(&self, code: &str) -> Option<DelegationRollup> {
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

    fn save(&self, item: &DelegationRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DelegationRollup> {
        self.inner.list()
    }
}

pub fn new_cached_delegation_rollup_store(
    inner: Rc<dyn DelegationRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DelegationRollupStore> {
    Rc::new(CachedDelegationRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DelegationRollup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects delegation rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRollupQuery {
    pub name: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DelegationRollupQuery {
    pub fn matches(&self, item: &DelegationRollup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DelegationRollup]) -> Vec<DelegationRollup> {
        let matched = collections::filter(items, |candidate: &DelegationRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_delegation_rollups(
    store: Rc<dyn DelegationRollupStore>,
    query: &DelegationRollupQuery,
) -> Vec<DelegationRollup> {
    query.apply(&store.list())
}
