use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback action item assignments by code.
pub trait FallbackActionItemAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackActionItemAssignment>;
    fn save(&self, item: &FallbackActionItemAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackActionItemAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackActionItemAssignmentStore {
    pub items: RefCell<BTreeMap<String, FallbackActionItemAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackActionItemAssignmentStore for MemoryFallbackActionItemAssignmentStore {
    fn find(&self, code: &str) -> Option<FallbackActionItemAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackActionItemAssignment) {
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

    fn list(&self) -> Vec<FallbackActionItemAssignment> {
        let mut out: Vec<FallbackActionItemAssignment> = Vec::new();
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

pub fn new_memory_fallback_action_item_assignment_store() -> Rc<dyn FallbackActionItemAssignmentStore> {
    Rc::new(MemoryFallbackActionItemAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackActionItemAssignmentStore {
    pub inner: Rc<dyn FallbackActionItemAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<FallbackActionItemAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackActionItemAssignmentStore for CachedFallbackActionItemAssignmentStore {
    fn find(&self, code: &str) -> Option<FallbackActionItemAssignment> {
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

    fn save(&self, item: &FallbackActionItemAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackActionItemAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_action_item_assignment_store(
    inner: Rc<dyn FallbackActionItemAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackActionItemAssignmentStore> {
    Rc::new(CachedFallbackActionItemAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackActionItemAssignment>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback action item assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemAssignmentQuery {
    pub label: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackActionItemAssignmentQuery {
    pub fn matches(&self, item: &FallbackActionItemAssignment) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackActionItemAssignment]) -> Vec<FallbackActionItemAssignment> {
        let matched = collections::filter(items, |candidate: &FallbackActionItemAssignment| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_action_item_assignments(
    store: Rc<dyn FallbackActionItemAssignmentStore>,
    query: &FallbackActionItemAssignmentQuery,
) -> Vec<FallbackActionItemAssignment> {
    query.apply(&store.list())
}
