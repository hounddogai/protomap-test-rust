use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled binding allocations by code.
pub trait ScheduledBindingAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledBindingAllocation>;
    fn save(&self, item: &ScheduledBindingAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledBindingAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledBindingAllocationStore {
    pub items: RefCell<BTreeMap<String, ScheduledBindingAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledBindingAllocationStore for MemoryScheduledBindingAllocationStore {
    fn find(&self, code: &str) -> Option<ScheduledBindingAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledBindingAllocation) {
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

    fn list(&self) -> Vec<ScheduledBindingAllocation> {
        let mut out: Vec<ScheduledBindingAllocation> = Vec::new();
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

pub fn new_memory_scheduled_binding_allocation_store() -> Rc<dyn ScheduledBindingAllocationStore> {
    Rc::new(MemoryScheduledBindingAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledBindingAllocationStore {
    pub inner: Rc<dyn ScheduledBindingAllocationStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledBindingAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledBindingAllocationStore for CachedScheduledBindingAllocationStore {
    fn find(&self, code: &str) -> Option<ScheduledBindingAllocation> {
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

    fn save(&self, item: &ScheduledBindingAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledBindingAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_binding_allocation_store(
    inner: Rc<dyn ScheduledBindingAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledBindingAllocationStore> {
    Rc::new(CachedScheduledBindingAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledBindingAllocation>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled binding allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingAllocationQuery {
    pub segment: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledBindingAllocationQuery {
    pub fn matches(&self, item: &ScheduledBindingAllocation) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledBindingAllocation]) -> Vec<ScheduledBindingAllocation> {
        let matched = collections::filter(items, |candidate: &ScheduledBindingAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_binding_allocations(
    store: Rc<dyn ScheduledBindingAllocationStore>,
    query: &ScheduledBindingAllocationQuery,
) -> Vec<ScheduledBindingAllocation> {
    query.apply(&store.list())
}
