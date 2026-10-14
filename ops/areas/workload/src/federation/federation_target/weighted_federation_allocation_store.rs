use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted federation allocations by code.
pub trait WeightedFederationAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedFederationAllocation>;
    fn save(&self, item: &WeightedFederationAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedFederationAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedFederationAllocationStore {
    pub items: RefCell<BTreeMap<String, WeightedFederationAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedFederationAllocationStore for MemoryWeightedFederationAllocationStore {
    fn find(&self, code: &str) -> Option<WeightedFederationAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedFederationAllocation) {
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

    fn list(&self) -> Vec<WeightedFederationAllocation> {
        let mut out: Vec<WeightedFederationAllocation> = Vec::new();
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

pub fn new_memory_weighted_federation_allocation_store() -> Rc<dyn WeightedFederationAllocationStore> {
    Rc::new(MemoryWeightedFederationAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedFederationAllocationStore {
    pub inner: Rc<dyn WeightedFederationAllocationStore>,
    pub cache: RefCell<cache::TtlCache<WeightedFederationAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedFederationAllocationStore for CachedWeightedFederationAllocationStore {
    fn find(&self, code: &str) -> Option<WeightedFederationAllocation> {
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

    fn save(&self, item: &WeightedFederationAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedFederationAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_federation_allocation_store(
    inner: Rc<dyn WeightedFederationAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedFederationAllocationStore> {
    Rc::new(CachedWeightedFederationAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedFederationAllocation>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted federation allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationAllocationQuery {
    pub scope: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedFederationAllocationQuery {
    pub fn matches(&self, item: &WeightedFederationAllocation) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedFederationAllocation]) -> Vec<WeightedFederationAllocation> {
        let matched = collections::filter(items, |candidate: &WeightedFederationAllocation| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_federation_allocations(
    store: Rc<dyn WeightedFederationAllocationStore>,
    query: &WeightedFederationAllocationQuery,
) -> Vec<WeightedFederationAllocation> {
    query.apply(&store.list())
}
