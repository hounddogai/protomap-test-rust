use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps limit allocations by code.
pub trait LimitAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LimitAllocation>;
    fn save(&self, item: &LimitAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LimitAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLimitAllocationStore {
    pub items: RefCell<BTreeMap<String, LimitAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl LimitAllocationStore for MemoryLimitAllocationStore {
    fn find(&self, code: &str) -> Option<LimitAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LimitAllocation) {
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

    fn list(&self) -> Vec<LimitAllocation> {
        let mut out: Vec<LimitAllocation> = Vec::new();
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

pub fn new_memory_limit_allocation_store() -> Rc<dyn LimitAllocationStore> {
    Rc::new(MemoryLimitAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLimitAllocationStore {
    pub inner: Rc<dyn LimitAllocationStore>,
    pub cache: RefCell<cache::TtlCache<LimitAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LimitAllocationStore for CachedLimitAllocationStore {
    fn find(&self, code: &str) -> Option<LimitAllocation> {
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

    fn save(&self, item: &LimitAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LimitAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_limit_allocation_store(
    inner: Rc<dyn LimitAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LimitAllocationStore> {
    Rc::new(CachedLimitAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LimitAllocation>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects limit allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitAllocationQuery {
    pub namespace: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LimitAllocationQuery {
    pub fn matches(&self, item: &LimitAllocation) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LimitAllocation]) -> Vec<LimitAllocation> {
        let matched = collections::filter(items, |candidate: &LimitAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_limit_allocations(
    store: Rc<dyn LimitAllocationStore>,
    query: &LimitAllocationQuery,
) -> Vec<LimitAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LimitAllocationStore> = new_memory_limit_allocation_store();
        let item = new_limit_allocation("code-879", "name-395");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
