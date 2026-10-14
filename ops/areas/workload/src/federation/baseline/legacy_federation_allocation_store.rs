use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy federation allocations by code.
pub trait LegacyFederationAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyFederationAllocation>;
    fn save(&self, item: &LegacyFederationAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyFederationAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyFederationAllocationStore {
    pub items: RefCell<BTreeMap<String, LegacyFederationAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyFederationAllocationStore for MemoryLegacyFederationAllocationStore {
    fn find(&self, code: &str) -> Option<LegacyFederationAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyFederationAllocation) {
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

    fn list(&self) -> Vec<LegacyFederationAllocation> {
        let mut out: Vec<LegacyFederationAllocation> = Vec::new();
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

pub fn new_memory_legacy_federation_allocation_store() -> Rc<dyn LegacyFederationAllocationStore> {
    Rc::new(MemoryLegacyFederationAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyFederationAllocationStore {
    pub inner: Rc<dyn LegacyFederationAllocationStore>,
    pub cache: RefCell<cache::TtlCache<LegacyFederationAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyFederationAllocationStore for CachedLegacyFederationAllocationStore {
    fn find(&self, code: &str) -> Option<LegacyFederationAllocation> {
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

    fn save(&self, item: &LegacyFederationAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyFederationAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_federation_allocation_store(
    inner: Rc<dyn LegacyFederationAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyFederationAllocationStore> {
    Rc::new(CachedLegacyFederationAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyFederationAllocation>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy federation allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAllocationQuery {
    pub name: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyFederationAllocationQuery {
    pub fn matches(&self, item: &LegacyFederationAllocation) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyFederationAllocation]) -> Vec<LegacyFederationAllocation> {
        let matched = collections::filter(items, |candidate: &LegacyFederationAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_federation_allocations(
    store: Rc<dyn LegacyFederationAllocationStore>,
    query: &LegacyFederationAllocationQuery,
) -> Vec<LegacyFederationAllocation> {
    query.apply(&store.list())
}
