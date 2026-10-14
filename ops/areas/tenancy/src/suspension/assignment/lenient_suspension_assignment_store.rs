use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient suspension assignments by code.
pub trait LenientSuspensionAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientSuspensionAssignment>;
    fn save(&self, item: &LenientSuspensionAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientSuspensionAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientSuspensionAssignmentStore {
    pub items: RefCell<BTreeMap<String, LenientSuspensionAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientSuspensionAssignmentStore for MemoryLenientSuspensionAssignmentStore {
    fn find(&self, code: &str) -> Option<LenientSuspensionAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientSuspensionAssignment) {
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

    fn list(&self) -> Vec<LenientSuspensionAssignment> {
        let mut out: Vec<LenientSuspensionAssignment> = Vec::new();
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

pub fn new_memory_lenient_suspension_assignment_store() -> Rc<dyn LenientSuspensionAssignmentStore> {
    Rc::new(MemoryLenientSuspensionAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientSuspensionAssignmentStore {
    pub inner: Rc<dyn LenientSuspensionAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<LenientSuspensionAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientSuspensionAssignmentStore for CachedLenientSuspensionAssignmentStore {
    fn find(&self, code: &str) -> Option<LenientSuspensionAssignment> {
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

    fn save(&self, item: &LenientSuspensionAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientSuspensionAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_suspension_assignment_store(
    inner: Rc<dyn LenientSuspensionAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientSuspensionAssignmentStore> {
    Rc::new(CachedLenientSuspensionAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientSuspensionAssignment>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient suspension assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAssignmentQuery {
    pub unit: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientSuspensionAssignmentQuery {
    pub fn matches(&self, item: &LenientSuspensionAssignment) -> bool {
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

    pub fn apply(&self, items: &[LenientSuspensionAssignment]) -> Vec<LenientSuspensionAssignment> {
        let matched = collections::filter(items, |candidate: &LenientSuspensionAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_suspension_assignments(
    store: Rc<dyn LenientSuspensionAssignmentStore>,
    query: &LenientSuspensionAssignmentQuery,
) -> Vec<LenientSuspensionAssignment> {
    query.apply(&store.list())
}
