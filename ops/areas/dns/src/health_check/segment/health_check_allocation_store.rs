use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps health check allocations by code.
pub trait HealthCheckAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<HealthCheckAllocation>;
    fn save(&self, item: &HealthCheckAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<HealthCheckAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHealthCheckAllocationStore {
    pub items: RefCell<BTreeMap<String, HealthCheckAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl HealthCheckAllocationStore for MemoryHealthCheckAllocationStore {
    fn find(&self, code: &str) -> Option<HealthCheckAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &HealthCheckAllocation) {
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

    fn list(&self) -> Vec<HealthCheckAllocation> {
        let mut out: Vec<HealthCheckAllocation> = Vec::new();
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

pub fn new_memory_health_check_allocation_store() -> Rc<dyn HealthCheckAllocationStore> {
    Rc::new(MemoryHealthCheckAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedHealthCheckAllocationStore {
    pub inner: Rc<dyn HealthCheckAllocationStore>,
    pub cache: RefCell<cache::TtlCache<HealthCheckAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl HealthCheckAllocationStore for CachedHealthCheckAllocationStore {
    fn find(&self, code: &str) -> Option<HealthCheckAllocation> {
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

    fn save(&self, item: &HealthCheckAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<HealthCheckAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_health_check_allocation_store(
    inner: Rc<dyn HealthCheckAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn HealthCheckAllocationStore> {
    Rc::new(CachedHealthCheckAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<HealthCheckAllocation>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects health check allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAllocationQuery {
    pub name: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl HealthCheckAllocationQuery {
    pub fn matches(&self, item: &HealthCheckAllocation) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[HealthCheckAllocation]) -> Vec<HealthCheckAllocation> {
        let matched = collections::filter(items, |candidate: &HealthCheckAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_health_check_allocations(
    store: Rc<dyn HealthCheckAllocationStore>,
    query: &HealthCheckAllocationQuery,
) -> Vec<HealthCheckAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn HealthCheckAllocationStore> = new_memory_health_check_allocation_store();
        let item = new_health_check_allocation("code-405", "name-402");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
