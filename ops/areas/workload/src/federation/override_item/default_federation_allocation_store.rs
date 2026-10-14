use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default federation allocations by code.
pub trait DefaultFederationAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultFederationAllocation>;
    fn save(&self, item: &DefaultFederationAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultFederationAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultFederationAllocationStore {
    pub items: RefCell<BTreeMap<String, DefaultFederationAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultFederationAllocationStore for MemoryDefaultFederationAllocationStore {
    fn find(&self, code: &str) -> Option<DefaultFederationAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultFederationAllocation) {
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

    fn list(&self) -> Vec<DefaultFederationAllocation> {
        let mut out: Vec<DefaultFederationAllocation> = Vec::new();
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

pub fn new_memory_default_federation_allocation_store() -> Rc<dyn DefaultFederationAllocationStore> {
    Rc::new(MemoryDefaultFederationAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultFederationAllocationStore {
    pub inner: Rc<dyn DefaultFederationAllocationStore>,
    pub cache: RefCell<cache::TtlCache<DefaultFederationAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultFederationAllocationStore for CachedDefaultFederationAllocationStore {
    fn find(&self, code: &str) -> Option<DefaultFederationAllocation> {
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

    fn save(&self, item: &DefaultFederationAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultFederationAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_default_federation_allocation_store(
    inner: Rc<dyn DefaultFederationAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultFederationAllocationStore> {
    Rc::new(CachedDefaultFederationAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultFederationAllocation>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default federation allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAllocationQuery {
    pub segment: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultFederationAllocationQuery {
    pub fn matches(&self, item: &DefaultFederationAllocation) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultFederationAllocation]) -> Vec<DefaultFederationAllocation> {
        let matched = collections::filter(items, |candidate: &DefaultFederationAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_federation_allocations(
    store: Rc<dyn DefaultFederationAllocationStore>,
    query: &DefaultFederationAllocationQuery,
) -> Vec<DefaultFederationAllocation> {
    query.apply(&store.list())
}
