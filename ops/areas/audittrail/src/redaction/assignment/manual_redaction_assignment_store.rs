use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual redaction assignments by code.
pub trait ManualRedactionAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualRedactionAssignment>;
    fn save(&self, item: &ManualRedactionAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualRedactionAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualRedactionAssignmentStore {
    pub items: RefCell<BTreeMap<String, ManualRedactionAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualRedactionAssignmentStore for MemoryManualRedactionAssignmentStore {
    fn find(&self, code: &str) -> Option<ManualRedactionAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualRedactionAssignment) {
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

    fn list(&self) -> Vec<ManualRedactionAssignment> {
        let mut out: Vec<ManualRedactionAssignment> = Vec::new();
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

pub fn new_memory_manual_redaction_assignment_store() -> Rc<dyn ManualRedactionAssignmentStore> {
    Rc::new(MemoryManualRedactionAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualRedactionAssignmentStore {
    pub inner: Rc<dyn ManualRedactionAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<ManualRedactionAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualRedactionAssignmentStore for CachedManualRedactionAssignmentStore {
    fn find(&self, code: &str) -> Option<ManualRedactionAssignment> {
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

    fn save(&self, item: &ManualRedactionAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualRedactionAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_manual_redaction_assignment_store(
    inner: Rc<dyn ManualRedactionAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualRedactionAssignmentStore> {
    Rc::new(CachedManualRedactionAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualRedactionAssignment>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual redaction assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionAssignmentQuery {
    pub locale: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualRedactionAssignmentQuery {
    pub fn matches(&self, item: &ManualRedactionAssignment) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualRedactionAssignment]) -> Vec<ManualRedactionAssignment> {
        let matched = collections::filter(items, |candidate: &ManualRedactionAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_redaction_assignments(
    store: Rc<dyn ManualRedactionAssignmentStore>,
    query: &ManualRedactionAssignmentQuery,
) -> Vec<ManualRedactionAssignment> {
    query.apply(&store.list())
}
