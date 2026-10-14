use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending suspension allocations by code.
pub trait PendingSuspensionAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingSuspensionAllocation>;
    fn save(&self, item: &PendingSuspensionAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingSuspensionAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingSuspensionAllocationStore {
    pub items: RefCell<BTreeMap<String, PendingSuspensionAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingSuspensionAllocationStore for MemoryPendingSuspensionAllocationStore {
    fn find(&self, code: &str) -> Option<PendingSuspensionAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingSuspensionAllocation) {
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

    fn list(&self) -> Vec<PendingSuspensionAllocation> {
        let mut out: Vec<PendingSuspensionAllocation> = Vec::new();
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

pub fn new_memory_pending_suspension_allocation_store() -> Rc<dyn PendingSuspensionAllocationStore> {
    Rc::new(MemoryPendingSuspensionAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingSuspensionAllocationStore {
    pub inner: Rc<dyn PendingSuspensionAllocationStore>,
    pub cache: RefCell<cache::TtlCache<PendingSuspensionAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingSuspensionAllocationStore for CachedPendingSuspensionAllocationStore {
    fn find(&self, code: &str) -> Option<PendingSuspensionAllocation> {
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

    fn save(&self, item: &PendingSuspensionAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingSuspensionAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_pending_suspension_allocation_store(
    inner: Rc<dyn PendingSuspensionAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingSuspensionAllocationStore> {
    Rc::new(CachedPendingSuspensionAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingSuspensionAllocation>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending suspension allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionAllocationQuery {
    pub namespace: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingSuspensionAllocationQuery {
    pub fn matches(&self, item: &PendingSuspensionAllocation) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[PendingSuspensionAllocation]) -> Vec<PendingSuspensionAllocation> {
        let matched = collections::filter(items, |candidate: &PendingSuspensionAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_suspension_allocations(
    store: Rc<dyn PendingSuspensionAllocationStore>,
    query: &PendingSuspensionAllocationQuery,
) -> Vec<PendingSuspensionAllocation> {
    query.apply(&store.list())
}
