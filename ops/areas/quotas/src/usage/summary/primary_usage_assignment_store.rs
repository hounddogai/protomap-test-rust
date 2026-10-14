use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary usage assignments by code.
pub trait PrimaryUsageAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryUsageAssignment>;
    fn save(&self, item: &PrimaryUsageAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryUsageAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryUsageAssignmentStore {
    pub items: RefCell<BTreeMap<String, PrimaryUsageAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryUsageAssignmentStore for MemoryPrimaryUsageAssignmentStore {
    fn find(&self, code: &str) -> Option<PrimaryUsageAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryUsageAssignment) {
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

    fn list(&self) -> Vec<PrimaryUsageAssignment> {
        let mut out: Vec<PrimaryUsageAssignment> = Vec::new();
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

pub fn new_memory_primary_usage_assignment_store() -> Rc<dyn PrimaryUsageAssignmentStore> {
    Rc::new(MemoryPrimaryUsageAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryUsageAssignmentStore {
    pub inner: Rc<dyn PrimaryUsageAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryUsageAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryUsageAssignmentStore for CachedPrimaryUsageAssignmentStore {
    fn find(&self, code: &str) -> Option<PrimaryUsageAssignment> {
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

    fn save(&self, item: &PrimaryUsageAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryUsageAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_primary_usage_assignment_store(
    inner: Rc<dyn PrimaryUsageAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryUsageAssignmentStore> {
    Rc::new(CachedPrimaryUsageAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryUsageAssignment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary usage assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAssignmentQuery {
    pub namespace: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryUsageAssignmentQuery {
    pub fn matches(&self, item: &PrimaryUsageAssignment) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryUsageAssignment]) -> Vec<PrimaryUsageAssignment> {
        let matched = collections::filter(items, |candidate: &PrimaryUsageAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_usage_assignments(
    store: Rc<dyn PrimaryUsageAssignmentStore>,
    query: &PrimaryUsageAssignmentQuery,
) -> Vec<PrimaryUsageAssignment> {
    query.apply(&store.list())
}
