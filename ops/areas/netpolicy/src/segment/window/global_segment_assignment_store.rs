use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global segment assignments by code.
pub trait GlobalSegmentAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalSegmentAssignment>;
    fn save(&self, item: &GlobalSegmentAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalSegmentAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalSegmentAssignmentStore {
    pub items: RefCell<BTreeMap<String, GlobalSegmentAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalSegmentAssignmentStore for MemoryGlobalSegmentAssignmentStore {
    fn find(&self, code: &str) -> Option<GlobalSegmentAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalSegmentAssignment) {
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

    fn list(&self) -> Vec<GlobalSegmentAssignment> {
        let mut out: Vec<GlobalSegmentAssignment> = Vec::new();
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

pub fn new_memory_global_segment_assignment_store() -> Rc<dyn GlobalSegmentAssignmentStore> {
    Rc::new(MemoryGlobalSegmentAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalSegmentAssignmentStore {
    pub inner: Rc<dyn GlobalSegmentAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<GlobalSegmentAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalSegmentAssignmentStore for CachedGlobalSegmentAssignmentStore {
    fn find(&self, code: &str) -> Option<GlobalSegmentAssignment> {
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

    fn save(&self, item: &GlobalSegmentAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalSegmentAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_global_segment_assignment_store(
    inner: Rc<dyn GlobalSegmentAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalSegmentAssignmentStore> {
    Rc::new(CachedGlobalSegmentAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalSegmentAssignment>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global segment assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentAssignmentQuery {
    pub name: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalSegmentAssignmentQuery {
    pub fn matches(&self, item: &GlobalSegmentAssignment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalSegmentAssignment]) -> Vec<GlobalSegmentAssignment> {
        let matched = collections::filter(items, |candidate: &GlobalSegmentAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_segment_assignments(
    store: Rc<dyn GlobalSegmentAssignmentStore>,
    query: &GlobalSegmentAssignmentQuery,
) -> Vec<GlobalSegmentAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalSegmentAssignmentStore> = new_memory_global_segment_assignment_store();
        let item = new_global_segment_assignment("code-3", "name-761");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
