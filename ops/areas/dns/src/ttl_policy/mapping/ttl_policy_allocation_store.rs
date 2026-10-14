use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps ttl policy allocations by code.
pub trait TtlPolicyAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TtlPolicyAllocation>;
    fn save(&self, item: &TtlPolicyAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TtlPolicyAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTtlPolicyAllocationStore {
    pub items: RefCell<BTreeMap<String, TtlPolicyAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl TtlPolicyAllocationStore for MemoryTtlPolicyAllocationStore {
    fn find(&self, code: &str) -> Option<TtlPolicyAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TtlPolicyAllocation) {
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

    fn list(&self) -> Vec<TtlPolicyAllocation> {
        let mut out: Vec<TtlPolicyAllocation> = Vec::new();
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

pub fn new_memory_ttl_policy_allocation_store() -> Rc<dyn TtlPolicyAllocationStore> {
    Rc::new(MemoryTtlPolicyAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTtlPolicyAllocationStore {
    pub inner: Rc<dyn TtlPolicyAllocationStore>,
    pub cache: RefCell<cache::TtlCache<TtlPolicyAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TtlPolicyAllocationStore for CachedTtlPolicyAllocationStore {
    fn find(&self, code: &str) -> Option<TtlPolicyAllocation> {
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

    fn save(&self, item: &TtlPolicyAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TtlPolicyAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_ttl_policy_allocation_store(
    inner: Rc<dyn TtlPolicyAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TtlPolicyAllocationStore> {
    Rc::new(CachedTtlPolicyAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TtlPolicyAllocation>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects ttl policy allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAllocationQuery {
    pub region: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TtlPolicyAllocationQuery {
    pub fn matches(&self, item: &TtlPolicyAllocation) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TtlPolicyAllocation]) -> Vec<TtlPolicyAllocation> {
        let matched = collections::filter(items, |candidate: &TtlPolicyAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_ttl_policy_allocations(
    store: Rc<dyn TtlPolicyAllocationStore>,
    query: &TtlPolicyAllocationQuery,
) -> Vec<TtlPolicyAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TtlPolicyAllocationStore> = new_memory_ttl_policy_allocation_store();
        let item = new_ttl_policy_allocation("code-355", "label-565");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
