use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item allocations by code.
pub trait ActionItemAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemAllocation>;
    fn save(&self, item: &ActionItemAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemAllocationStore {
    pub items: RefCell<BTreeMap<String, ActionItemAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemAllocationStore for MemoryActionItemAllocationStore {
    fn find(&self, code: &str) -> Option<ActionItemAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemAllocation) {
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

    fn list(&self) -> Vec<ActionItemAllocation> {
        let mut out: Vec<ActionItemAllocation> = Vec::new();
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

pub fn new_memory_action_item_allocation_store() -> Rc<dyn ActionItemAllocationStore> {
    Rc::new(MemoryActionItemAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemAllocationStore {
    pub inner: Rc<dyn ActionItemAllocationStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemAllocationStore for CachedActionItemAllocationStore {
    fn find(&self, code: &str) -> Option<ActionItemAllocation> {
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

    fn save(&self, item: &ActionItemAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_allocation_store(
    inner: Rc<dyn ActionItemAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemAllocationStore> {
    Rc::new(CachedActionItemAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemAllocation>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAllocationQuery {
    pub namespace: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemAllocationQuery {
    pub fn matches(&self, item: &ActionItemAllocation) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ActionItemAllocation]) -> Vec<ActionItemAllocation> {
        let matched = collections::filter(items, |candidate: &ActionItemAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_allocations(
    store: Rc<dyn ActionItemAllocationStore>,
    query: &ActionItemAllocationQuery,
) -> Vec<ActionItemAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ActionItemAllocationStore> = new_memory_action_item_allocation_store();
        let item = new_action_item_allocation("code-922", "title-595");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
