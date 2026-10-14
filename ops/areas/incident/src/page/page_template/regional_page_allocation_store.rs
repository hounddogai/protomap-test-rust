use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional page allocations by code.
pub trait RegionalPageAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalPageAllocation>;
    fn save(&self, item: &RegionalPageAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalPageAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalPageAllocationStore {
    pub items: RefCell<BTreeMap<String, RegionalPageAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalPageAllocationStore for MemoryRegionalPageAllocationStore {
    fn find(&self, code: &str) -> Option<RegionalPageAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalPageAllocation) {
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

    fn list(&self) -> Vec<RegionalPageAllocation> {
        let mut out: Vec<RegionalPageAllocation> = Vec::new();
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

pub fn new_memory_regional_page_allocation_store() -> Rc<dyn RegionalPageAllocationStore> {
    Rc::new(MemoryRegionalPageAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalPageAllocationStore {
    pub inner: Rc<dyn RegionalPageAllocationStore>,
    pub cache: RefCell<cache::TtlCache<RegionalPageAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalPageAllocationStore for CachedRegionalPageAllocationStore {
    fn find(&self, code: &str) -> Option<RegionalPageAllocation> {
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

    fn save(&self, item: &RegionalPageAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalPageAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_regional_page_allocation_store(
    inner: Rc<dyn RegionalPageAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalPageAllocationStore> {
    Rc::new(CachedRegionalPageAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalPageAllocation>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional page allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageAllocationQuery {
    pub name: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalPageAllocationQuery {
    pub fn matches(&self, item: &RegionalPageAllocation) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalPageAllocation]) -> Vec<RegionalPageAllocation> {
        let matched = collections::filter(items, |candidate: &RegionalPageAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_page_allocations(
    store: Rc<dyn RegionalPageAllocationStore>,
    query: &RegionalPageAllocationQuery,
) -> Vec<RegionalPageAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalPageAllocationStore> = new_memory_regional_page_allocation_store();
        let item = new_regional_page_allocation("code-409", "name-470");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
