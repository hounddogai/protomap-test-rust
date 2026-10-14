use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary canary assignments by code.
pub trait PrimaryCanaryAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryCanaryAssignment>;
    fn save(&self, item: &PrimaryCanaryAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryCanaryAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryCanaryAssignmentStore {
    pub items: RefCell<BTreeMap<String, PrimaryCanaryAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryCanaryAssignmentStore for MemoryPrimaryCanaryAssignmentStore {
    fn find(&self, code: &str) -> Option<PrimaryCanaryAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryCanaryAssignment) {
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

    fn list(&self) -> Vec<PrimaryCanaryAssignment> {
        let mut out: Vec<PrimaryCanaryAssignment> = Vec::new();
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

pub fn new_memory_primary_canary_assignment_store() -> Rc<dyn PrimaryCanaryAssignmentStore> {
    Rc::new(MemoryPrimaryCanaryAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryCanaryAssignmentStore {
    pub inner: Rc<dyn PrimaryCanaryAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryCanaryAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryCanaryAssignmentStore for CachedPrimaryCanaryAssignmentStore {
    fn find(&self, code: &str) -> Option<PrimaryCanaryAssignment> {
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

    fn save(&self, item: &PrimaryCanaryAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryCanaryAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_primary_canary_assignment_store(
    inner: Rc<dyn PrimaryCanaryAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryCanaryAssignmentStore> {
    Rc::new(CachedPrimaryCanaryAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryCanaryAssignment>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary canary assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignmentQuery {
    pub name: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryCanaryAssignmentQuery {
    pub fn matches(&self, item: &PrimaryCanaryAssignment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryCanaryAssignment]) -> Vec<PrimaryCanaryAssignment> {
        let matched = collections::filter(items, |candidate: &PrimaryCanaryAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_canary_assignments(
    store: Rc<dyn PrimaryCanaryAssignmentStore>,
    query: &PrimaryCanaryAssignmentQuery,
) -> Vec<PrimaryCanaryAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryCanaryAssignmentStore> = new_memory_primary_canary_assignment_store();
        let item = new_primary_canary_assignment("code-878", "name-102");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
