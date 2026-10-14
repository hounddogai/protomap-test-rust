use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional canary allocations by code.
pub trait RegionalCanaryAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalCanaryAllocation>;
    fn save(&self, item: &RegionalCanaryAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalCanaryAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalCanaryAllocationStore {
    pub items: RefCell<BTreeMap<String, RegionalCanaryAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalCanaryAllocationStore for MemoryRegionalCanaryAllocationStore {
    fn find(&self, code: &str) -> Option<RegionalCanaryAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalCanaryAllocation) {
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

    fn list(&self) -> Vec<RegionalCanaryAllocation> {
        let mut out: Vec<RegionalCanaryAllocation> = Vec::new();
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

pub fn new_memory_regional_canary_allocation_store() -> Rc<dyn RegionalCanaryAllocationStore> {
    Rc::new(MemoryRegionalCanaryAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalCanaryAllocationStore {
    pub inner: Rc<dyn RegionalCanaryAllocationStore>,
    pub cache: RefCell<cache::TtlCache<RegionalCanaryAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalCanaryAllocationStore for CachedRegionalCanaryAllocationStore {
    fn find(&self, code: &str) -> Option<RegionalCanaryAllocation> {
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

    fn save(&self, item: &RegionalCanaryAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalCanaryAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_regional_canary_allocation_store(
    inner: Rc<dyn RegionalCanaryAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalCanaryAllocationStore> {
    Rc::new(CachedRegionalCanaryAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalCanaryAllocation>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional canary allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAllocationQuery {
    pub channel: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalCanaryAllocationQuery {
    pub fn matches(&self, item: &RegionalCanaryAllocation) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[RegionalCanaryAllocation]) -> Vec<RegionalCanaryAllocation> {
        let matched = collections::filter(items, |candidate: &RegionalCanaryAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_canary_allocations(
    store: Rc<dyn RegionalCanaryAllocationStore>,
    query: &RegionalCanaryAllocationQuery,
) -> Vec<RegionalCanaryAllocation> {
    query.apply(&store.list())
}
