use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global release assignments by code.
pub trait GlobalReleaseAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalReleaseAssignment>;
    fn save(&self, item: &GlobalReleaseAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalReleaseAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalReleaseAssignmentStore {
    pub items: RefCell<BTreeMap<String, GlobalReleaseAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalReleaseAssignmentStore for MemoryGlobalReleaseAssignmentStore {
    fn find(&self, code: &str) -> Option<GlobalReleaseAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalReleaseAssignment) {
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

    fn list(&self) -> Vec<GlobalReleaseAssignment> {
        let mut out: Vec<GlobalReleaseAssignment> = Vec::new();
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

pub fn new_memory_global_release_assignment_store() -> Rc<dyn GlobalReleaseAssignmentStore> {
    Rc::new(MemoryGlobalReleaseAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalReleaseAssignmentStore {
    pub inner: Rc<dyn GlobalReleaseAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<GlobalReleaseAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalReleaseAssignmentStore for CachedGlobalReleaseAssignmentStore {
    fn find(&self, code: &str) -> Option<GlobalReleaseAssignment> {
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

    fn save(&self, item: &GlobalReleaseAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalReleaseAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_global_release_assignment_store(
    inner: Rc<dyn GlobalReleaseAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalReleaseAssignmentStore> {
    Rc::new(CachedGlobalReleaseAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalReleaseAssignment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global release assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAssignmentQuery {
    pub name: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalReleaseAssignmentQuery {
    pub fn matches(&self, item: &GlobalReleaseAssignment) -> bool {
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

    pub fn apply(&self, items: &[GlobalReleaseAssignment]) -> Vec<GlobalReleaseAssignment> {
        let matched = collections::filter(items, |candidate: &GlobalReleaseAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_release_assignments(
    store: Rc<dyn GlobalReleaseAssignmentStore>,
    query: &GlobalReleaseAssignmentQuery,
) -> Vec<GlobalReleaseAssignment> {
    query.apply(&store.list())
}
