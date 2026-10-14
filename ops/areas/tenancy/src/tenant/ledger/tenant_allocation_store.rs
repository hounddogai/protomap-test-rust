use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tenant allocations by code.
pub trait TenantAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TenantAllocation>;
    fn save(&self, item: &TenantAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TenantAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTenantAllocationStore {
    pub items: RefCell<BTreeMap<String, TenantAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl TenantAllocationStore for MemoryTenantAllocationStore {
    fn find(&self, code: &str) -> Option<TenantAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TenantAllocation) {
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

    fn list(&self) -> Vec<TenantAllocation> {
        let mut out: Vec<TenantAllocation> = Vec::new();
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

pub fn new_memory_tenant_allocation_store() -> Rc<dyn TenantAllocationStore> {
    Rc::new(MemoryTenantAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTenantAllocationStore {
    pub inner: Rc<dyn TenantAllocationStore>,
    pub cache: RefCell<cache::TtlCache<TenantAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TenantAllocationStore for CachedTenantAllocationStore {
    fn find(&self, code: &str) -> Option<TenantAllocation> {
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

    fn save(&self, item: &TenantAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TenantAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_tenant_allocation_store(
    inner: Rc<dyn TenantAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TenantAllocationStore> {
    Rc::new(CachedTenantAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TenantAllocation>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tenant allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAllocationQuery {
    pub scope: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TenantAllocationQuery {
    pub fn matches(&self, item: &TenantAllocation) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TenantAllocation]) -> Vec<TenantAllocation> {
        let matched = collections::filter(items, |candidate: &TenantAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tenant_allocations(
    store: Rc<dyn TenantAllocationStore>,
    query: &TenantAllocationQuery,
) -> Vec<TenantAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TenantAllocationStore> = new_memory_tenant_allocation_store();
        let item = new_tenant_allocation("code-247", "label-210");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
