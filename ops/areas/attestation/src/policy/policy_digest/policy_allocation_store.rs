use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps policy allocations by code.
pub trait PolicyAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PolicyAllocation>;
    fn save(&self, item: &PolicyAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PolicyAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPolicyAllocationStore {
    pub items: RefCell<BTreeMap<String, PolicyAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl PolicyAllocationStore for MemoryPolicyAllocationStore {
    fn find(&self, code: &str) -> Option<PolicyAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PolicyAllocation) {
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

    fn list(&self) -> Vec<PolicyAllocation> {
        let mut out: Vec<PolicyAllocation> = Vec::new();
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

pub fn new_memory_policy_allocation_store() -> Rc<dyn PolicyAllocationStore> {
    Rc::new(MemoryPolicyAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPolicyAllocationStore {
    pub inner: Rc<dyn PolicyAllocationStore>,
    pub cache: RefCell<cache::TtlCache<PolicyAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PolicyAllocationStore for CachedPolicyAllocationStore {
    fn find(&self, code: &str) -> Option<PolicyAllocation> {
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

    fn save(&self, item: &PolicyAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PolicyAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_policy_allocation_store(
    inner: Rc<dyn PolicyAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PolicyAllocationStore> {
    Rc::new(CachedPolicyAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PolicyAllocation>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects policy allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAllocationQuery {
    pub category: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PolicyAllocationQuery {
    pub fn matches(&self, item: &PolicyAllocation) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PolicyAllocation]) -> Vec<PolicyAllocation> {
        let matched = collections::filter(items, |candidate: &PolicyAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_policy_allocations(
    store: Rc<dyn PolicyAllocationStore>,
    query: &PolicyAllocationQuery,
) -> Vec<PolicyAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PolicyAllocationStore> = new_memory_policy_allocation_store();
        let item = new_policy_allocation("code-633", "name-199");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
