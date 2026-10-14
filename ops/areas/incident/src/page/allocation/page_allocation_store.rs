use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps page allocations by code.
pub trait PageAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PageAllocation>;
    fn save(&self, item: &PageAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PageAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPageAllocationStore {
    pub items: RefCell<BTreeMap<String, PageAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl PageAllocationStore for MemoryPageAllocationStore {
    fn find(&self, code: &str) -> Option<PageAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PageAllocation) {
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

    fn list(&self) -> Vec<PageAllocation> {
        let mut out: Vec<PageAllocation> = Vec::new();
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

pub fn new_memory_page_allocation_store() -> Rc<dyn PageAllocationStore> {
    Rc::new(MemoryPageAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPageAllocationStore {
    pub inner: Rc<dyn PageAllocationStore>,
    pub cache: RefCell<cache::TtlCache<PageAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PageAllocationStore for CachedPageAllocationStore {
    fn find(&self, code: &str) -> Option<PageAllocation> {
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

    fn save(&self, item: &PageAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PageAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_page_allocation_store(
    inner: Rc<dyn PageAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PageAllocationStore> {
    Rc::new(CachedPageAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PageAllocation>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects page allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAllocationQuery {
    pub scope: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PageAllocationQuery {
    pub fn matches(&self, item: &PageAllocation) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PageAllocation]) -> Vec<PageAllocation> {
        let matched = collections::filter(items, |candidate: &PageAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_page_allocations(store: Rc<dyn PageAllocationStore>, query: &PageAllocationQuery) -> Vec<PageAllocation> {
    query.apply(&store.list())
}
