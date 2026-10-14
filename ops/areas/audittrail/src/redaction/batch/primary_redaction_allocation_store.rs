use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary redaction allocations by code.
pub trait PrimaryRedactionAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryRedactionAllocation>;
    fn save(&self, item: &PrimaryRedactionAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryRedactionAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryRedactionAllocationStore {
    pub items: RefCell<BTreeMap<String, PrimaryRedactionAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryRedactionAllocationStore for MemoryPrimaryRedactionAllocationStore {
    fn find(&self, code: &str) -> Option<PrimaryRedactionAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryRedactionAllocation) {
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

    fn list(&self) -> Vec<PrimaryRedactionAllocation> {
        let mut out: Vec<PrimaryRedactionAllocation> = Vec::new();
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

pub fn new_memory_primary_redaction_allocation_store() -> Rc<dyn PrimaryRedactionAllocationStore> {
    Rc::new(MemoryPrimaryRedactionAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryRedactionAllocationStore {
    pub inner: Rc<dyn PrimaryRedactionAllocationStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryRedactionAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryRedactionAllocationStore for CachedPrimaryRedactionAllocationStore {
    fn find(&self, code: &str) -> Option<PrimaryRedactionAllocation> {
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

    fn save(&self, item: &PrimaryRedactionAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryRedactionAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_primary_redaction_allocation_store(
    inner: Rc<dyn PrimaryRedactionAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryRedactionAllocationStore> {
    Rc::new(CachedPrimaryRedactionAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryRedactionAllocation>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary redaction allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionAllocationQuery {
    pub unit: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryRedactionAllocationQuery {
    pub fn matches(&self, item: &PrimaryRedactionAllocation) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryRedactionAllocation]) -> Vec<PrimaryRedactionAllocation> {
        let matched = collections::filter(items, |candidate: &PrimaryRedactionAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_redaction_allocations(
    store: Rc<dyn PrimaryRedactionAllocationStore>,
    query: &PrimaryRedactionAllocationQuery,
) -> Vec<PrimaryRedactionAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryRedactionAllocationStore> = new_memory_primary_redaction_allocation_store();
        let item = new_primary_redaction_allocation("code-830", "title-723");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
