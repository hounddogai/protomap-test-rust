use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps verifier assignments by code.
pub trait VerifierAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<VerifierAssignment>;
    fn save(&self, item: &VerifierAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<VerifierAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryVerifierAssignmentStore {
    pub items: RefCell<BTreeMap<String, VerifierAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl VerifierAssignmentStore for MemoryVerifierAssignmentStore {
    fn find(&self, code: &str) -> Option<VerifierAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &VerifierAssignment) {
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

    fn list(&self) -> Vec<VerifierAssignment> {
        let mut out: Vec<VerifierAssignment> = Vec::new();
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

pub fn new_memory_verifier_assignment_store() -> Rc<dyn VerifierAssignmentStore> {
    Rc::new(MemoryVerifierAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedVerifierAssignmentStore {
    pub inner: Rc<dyn VerifierAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<VerifierAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl VerifierAssignmentStore for CachedVerifierAssignmentStore {
    fn find(&self, code: &str) -> Option<VerifierAssignment> {
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

    fn save(&self, item: &VerifierAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<VerifierAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_verifier_assignment_store(
    inner: Rc<dyn VerifierAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn VerifierAssignmentStore> {
    Rc::new(CachedVerifierAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<VerifierAssignment>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects verifier assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAssignmentQuery {
    pub label: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl VerifierAssignmentQuery {
    pub fn matches(&self, item: &VerifierAssignment) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[VerifierAssignment]) -> Vec<VerifierAssignment> {
        let matched = collections::filter(items, |candidate: &VerifierAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_verifier_assignments(
    store: Rc<dyn VerifierAssignmentStore>,
    query: &VerifierAssignmentQuery,
) -> Vec<VerifierAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn VerifierAssignmentStore> = new_memory_verifier_assignment_store();
        let item = new_verifier_assignment("code-802", "label-796");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
