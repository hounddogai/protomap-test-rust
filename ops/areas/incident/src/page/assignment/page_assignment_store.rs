use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps page assignments by code.
pub trait PageAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PageAssignment>;
    fn save(&self, item: &PageAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PageAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPageAssignmentStore {
    pub items: RefCell<BTreeMap<String, PageAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl PageAssignmentStore for MemoryPageAssignmentStore {
    fn find(&self, code: &str) -> Option<PageAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PageAssignment) {
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

    fn list(&self) -> Vec<PageAssignment> {
        let mut out: Vec<PageAssignment> = Vec::new();
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

pub fn new_memory_page_assignment_store() -> Rc<dyn PageAssignmentStore> {
    Rc::new(MemoryPageAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPageAssignmentStore {
    pub inner: Rc<dyn PageAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<PageAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PageAssignmentStore for CachedPageAssignmentStore {
    fn find(&self, code: &str) -> Option<PageAssignment> {
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

    fn save(&self, item: &PageAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PageAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_page_assignment_store(
    inner: Rc<dyn PageAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PageAssignmentStore> {
    Rc::new(CachedPageAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PageAssignment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects page assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAssignmentQuery {
    pub name: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PageAssignmentQuery {
    pub fn matches(&self, item: &PageAssignment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PageAssignment]) -> Vec<PageAssignment> {
        let matched = collections::filter(items, |candidate: &PageAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_page_assignments(store: Rc<dyn PageAssignmentStore>, query: &PageAssignmentQuery) -> Vec<PageAssignment> {
    query.apply(&store.list())
}
