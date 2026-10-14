use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps failover allocations by code.
pub trait FailoverAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FailoverAllocation>;
    fn save(&self, item: &FailoverAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FailoverAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFailoverAllocationStore {
    pub items: RefCell<BTreeMap<String, FailoverAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl FailoverAllocationStore for MemoryFailoverAllocationStore {
    fn find(&self, code: &str) -> Option<FailoverAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FailoverAllocation) {
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

    fn list(&self) -> Vec<FailoverAllocation> {
        let mut out: Vec<FailoverAllocation> = Vec::new();
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

pub fn new_memory_failover_allocation_store() -> Rc<dyn FailoverAllocationStore> {
    Rc::new(MemoryFailoverAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFailoverAllocationStore {
    pub inner: Rc<dyn FailoverAllocationStore>,
    pub cache: RefCell<cache::TtlCache<FailoverAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FailoverAllocationStore for CachedFailoverAllocationStore {
    fn find(&self, code: &str) -> Option<FailoverAllocation> {
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

    fn save(&self, item: &FailoverAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FailoverAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_failover_allocation_store(
    inner: Rc<dyn FailoverAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FailoverAllocationStore> {
    Rc::new(CachedFailoverAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FailoverAllocation>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects failover allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAllocationQuery {
    pub region: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FailoverAllocationQuery {
    pub fn matches(&self, item: &FailoverAllocation) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FailoverAllocation]) -> Vec<FailoverAllocation> {
        let matched = collections::filter(items, |candidate: &FailoverAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_failover_allocations(
    store: Rc<dyn FailoverAllocationStore>,
    query: &FailoverAllocationQuery,
) -> Vec<FailoverAllocation> {
    query.apply(&store.list())
}
