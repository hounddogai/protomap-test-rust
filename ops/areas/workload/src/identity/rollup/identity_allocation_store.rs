use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity allocations by code.
pub trait IdentityAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityAllocation>;
    fn save(&self, item: &IdentityAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityAllocationStore {
    pub items: RefCell<BTreeMap<String, IdentityAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityAllocationStore for MemoryIdentityAllocationStore {
    fn find(&self, code: &str) -> Option<IdentityAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityAllocation) {
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

    fn list(&self) -> Vec<IdentityAllocation> {
        let mut out: Vec<IdentityAllocation> = Vec::new();
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

pub fn new_memory_identity_allocation_store() -> Rc<dyn IdentityAllocationStore> {
    Rc::new(MemoryIdentityAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityAllocationStore {
    pub inner: Rc<dyn IdentityAllocationStore>,
    pub cache: RefCell<cache::TtlCache<IdentityAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityAllocationStore for CachedIdentityAllocationStore {
    fn find(&self, code: &str) -> Option<IdentityAllocation> {
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

    fn save(&self, item: &IdentityAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_identity_allocation_store(
    inner: Rc<dyn IdentityAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityAllocationStore> {
    Rc::new(CachedIdentityAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityAllocation>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAllocationQuery {
    pub scope: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityAllocationQuery {
    pub fn matches(&self, item: &IdentityAllocation) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IdentityAllocation]) -> Vec<IdentityAllocation> {
        let matched = collections::filter(items, |candidate: &IdentityAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_allocations(
    store: Rc<dyn IdentityAllocationStore>,
    query: &IdentityAllocationQuery,
) -> Vec<IdentityAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IdentityAllocationStore> = new_memory_identity_allocation_store();
        let item = new_identity_allocation("code-282", "label-697");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
