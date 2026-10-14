use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps federation groups by code.
pub trait FederationGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FederationGroup>;
    fn save(&self, item: &FederationGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FederationGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFederationGroupStore {
    pub items: RefCell<BTreeMap<String, FederationGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl FederationGroupStore for MemoryFederationGroupStore {
    fn find(&self, code: &str) -> Option<FederationGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FederationGroup) {
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

    fn list(&self) -> Vec<FederationGroup> {
        let mut out: Vec<FederationGroup> = Vec::new();
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

pub fn new_memory_federation_group_store() -> Rc<dyn FederationGroupStore> {
    Rc::new(MemoryFederationGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFederationGroupStore {
    pub inner: Rc<dyn FederationGroupStore>,
    pub cache: RefCell<cache::TtlCache<FederationGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FederationGroupStore for CachedFederationGroupStore {
    fn find(&self, code: &str) -> Option<FederationGroup> {
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

    fn save(&self, item: &FederationGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FederationGroup> {
        self.inner.list()
    }
}

pub fn new_cached_federation_group_store(
    inner: Rc<dyn FederationGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FederationGroupStore> {
    Rc::new(CachedFederationGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FederationGroup>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects federation groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationGroupQuery {
    pub scope: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FederationGroupQuery {
    pub fn matches(&self, item: &FederationGroup) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[FederationGroup]) -> Vec<FederationGroup> {
        let matched = collections::filter(items, |candidate: &FederationGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_federation_groups(
    store: Rc<dyn FederationGroupStore>,
    query: &FederationGroupQuery,
) -> Vec<FederationGroup> {
    query.apply(&store.list())
}
