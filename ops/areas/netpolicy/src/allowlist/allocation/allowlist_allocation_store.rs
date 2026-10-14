use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps allowlist allocations by code.
pub trait AllowlistAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AllowlistAllocation>;
    fn save(&self, item: &AllowlistAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AllowlistAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAllowlistAllocationStore {
    pub items: RefCell<BTreeMap<String, AllowlistAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl AllowlistAllocationStore for MemoryAllowlistAllocationStore {
    fn find(&self, code: &str) -> Option<AllowlistAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AllowlistAllocation) {
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

    fn list(&self) -> Vec<AllowlistAllocation> {
        let mut out: Vec<AllowlistAllocation> = Vec::new();
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

pub fn new_memory_allowlist_allocation_store() -> Rc<dyn AllowlistAllocationStore> {
    Rc::new(MemoryAllowlistAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAllowlistAllocationStore {
    pub inner: Rc<dyn AllowlistAllocationStore>,
    pub cache: RefCell<cache::TtlCache<AllowlistAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AllowlistAllocationStore for CachedAllowlistAllocationStore {
    fn find(&self, code: &str) -> Option<AllowlistAllocation> {
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

    fn save(&self, item: &AllowlistAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AllowlistAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_allowlist_allocation_store(
    inner: Rc<dyn AllowlistAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AllowlistAllocationStore> {
    Rc::new(CachedAllowlistAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AllowlistAllocation>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects allowlist allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAllocationQuery {
    pub source: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AllowlistAllocationQuery {
    pub fn matches(&self, item: &AllowlistAllocation) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[AllowlistAllocation]) -> Vec<AllowlistAllocation> {
        let matched = collections::filter(items, |candidate: &AllowlistAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_allowlist_allocations(
    store: Rc<dyn AllowlistAllocationStore>,
    query: &AllowlistAllocationQuery,
) -> Vec<AllowlistAllocation> {
    query.apply(&store.list())
}
