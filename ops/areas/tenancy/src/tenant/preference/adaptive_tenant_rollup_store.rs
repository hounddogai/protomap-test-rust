use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive tenant rollups by code.
pub trait AdaptiveTenantRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveTenantRollup>;
    fn save(&self, item: &AdaptiveTenantRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveTenantRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveTenantRollupStore {
    pub items: RefCell<BTreeMap<String, AdaptiveTenantRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveTenantRollupStore for MemoryAdaptiveTenantRollupStore {
    fn find(&self, code: &str) -> Option<AdaptiveTenantRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveTenantRollup) {
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

    fn list(&self) -> Vec<AdaptiveTenantRollup> {
        let mut out: Vec<AdaptiveTenantRollup> = Vec::new();
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

pub fn new_memory_adaptive_tenant_rollup_store() -> Rc<dyn AdaptiveTenantRollupStore> {
    Rc::new(MemoryAdaptiveTenantRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveTenantRollupStore {
    pub inner: Rc<dyn AdaptiveTenantRollupStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveTenantRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveTenantRollupStore for CachedAdaptiveTenantRollupStore {
    fn find(&self, code: &str) -> Option<AdaptiveTenantRollup> {
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

    fn save(&self, item: &AdaptiveTenantRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveTenantRollup> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_tenant_rollup_store(
    inner: Rc<dyn AdaptiveTenantRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveTenantRollupStore> {
    Rc::new(CachedAdaptiveTenantRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveTenantRollup>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive tenant rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantRollupQuery {
    pub locale: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveTenantRollupQuery {
    pub fn matches(&self, item: &AdaptiveTenantRollup) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveTenantRollup]) -> Vec<AdaptiveTenantRollup> {
        let matched = collections::filter(items, |candidate: &AdaptiveTenantRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_tenant_rollups(
    store: Rc<dyn AdaptiveTenantRollupStore>,
    query: &AdaptiveTenantRollupQuery,
) -> Vec<AdaptiveTenantRollup> {
    query.apply(&store.list())
}
