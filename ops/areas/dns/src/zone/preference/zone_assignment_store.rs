use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps zone assignments by code.
pub trait ZoneAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ZoneAssignment>;
    fn save(&self, item: &ZoneAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ZoneAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryZoneAssignmentStore {
    pub items: RefCell<BTreeMap<String, ZoneAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl ZoneAssignmentStore for MemoryZoneAssignmentStore {
    fn find(&self, code: &str) -> Option<ZoneAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ZoneAssignment) {
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

    fn list(&self) -> Vec<ZoneAssignment> {
        let mut out: Vec<ZoneAssignment> = Vec::new();
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

pub fn new_memory_zone_assignment_store() -> Rc<dyn ZoneAssignmentStore> {
    Rc::new(MemoryZoneAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedZoneAssignmentStore {
    pub inner: Rc<dyn ZoneAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<ZoneAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ZoneAssignmentStore for CachedZoneAssignmentStore {
    fn find(&self, code: &str) -> Option<ZoneAssignment> {
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

    fn save(&self, item: &ZoneAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ZoneAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_zone_assignment_store(
    inner: Rc<dyn ZoneAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ZoneAssignmentStore> {
    Rc::new(CachedZoneAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ZoneAssignment>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects zone assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAssignmentQuery {
    pub source: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ZoneAssignmentQuery {
    pub fn matches(&self, item: &ZoneAssignment) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ZoneAssignment]) -> Vec<ZoneAssignment> {
        let matched = collections::filter(items, |candidate: &ZoneAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_zone_assignments(store: Rc<dyn ZoneAssignmentStore>, query: &ZoneAssignmentQuery) -> Vec<ZoneAssignment> {
    query.apply(&store.list())
}
