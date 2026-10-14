use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict tenant allocations by code.
pub trait StrictTenantAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictTenantAllocation>;
    fn save(&self, item: &StrictTenantAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictTenantAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictTenantAllocationStore {
    pub items: RefCell<BTreeMap<String, StrictTenantAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictTenantAllocationStore for MemoryStrictTenantAllocationStore {
    fn find(&self, code: &str) -> Option<StrictTenantAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictTenantAllocation) {
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

    fn list(&self) -> Vec<StrictTenantAllocation> {
        let mut out: Vec<StrictTenantAllocation> = Vec::new();
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

pub fn new_memory_strict_tenant_allocation_store() -> Rc<dyn StrictTenantAllocationStore> {
    Rc::new(MemoryStrictTenantAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictTenantAllocationStore {
    pub inner: Rc<dyn StrictTenantAllocationStore>,
    pub cache: RefCell<cache::TtlCache<StrictTenantAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictTenantAllocationStore for CachedStrictTenantAllocationStore {
    fn find(&self, code: &str) -> Option<StrictTenantAllocation> {
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

    fn save(&self, item: &StrictTenantAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictTenantAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_strict_tenant_allocation_store(
    inner: Rc<dyn StrictTenantAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictTenantAllocationStore> {
    Rc::new(CachedStrictTenantAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictTenantAllocation>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict tenant allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAllocationQuery {
    pub unit: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictTenantAllocationQuery {
    pub fn matches(&self, item: &StrictTenantAllocation) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictTenantAllocation]) -> Vec<StrictTenantAllocation> {
        let matched = collections::filter(items, |candidate: &StrictTenantAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_tenant_allocations(
    store: Rc<dyn StrictTenantAllocationStore>,
    query: &StrictTenantAllocationQuery,
) -> Vec<StrictTenantAllocation> {
    query.apply(&store.list())
}
