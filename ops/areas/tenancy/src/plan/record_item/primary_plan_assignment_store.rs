use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary plan assignments by code.
pub trait PrimaryPlanAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryPlanAssignment>;
    fn save(&self, item: &PrimaryPlanAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryPlanAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryPlanAssignmentStore {
    pub items: RefCell<BTreeMap<String, PrimaryPlanAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryPlanAssignmentStore for MemoryPrimaryPlanAssignmentStore {
    fn find(&self, code: &str) -> Option<PrimaryPlanAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryPlanAssignment) {
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

    fn list(&self) -> Vec<PrimaryPlanAssignment> {
        let mut out: Vec<PrimaryPlanAssignment> = Vec::new();
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

pub fn new_memory_primary_plan_assignment_store() -> Rc<dyn PrimaryPlanAssignmentStore> {
    Rc::new(MemoryPrimaryPlanAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryPlanAssignmentStore {
    pub inner: Rc<dyn PrimaryPlanAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryPlanAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryPlanAssignmentStore for CachedPrimaryPlanAssignmentStore {
    fn find(&self, code: &str) -> Option<PrimaryPlanAssignment> {
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

    fn save(&self, item: &PrimaryPlanAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryPlanAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_primary_plan_assignment_store(
    inner: Rc<dyn PrimaryPlanAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryPlanAssignmentStore> {
    Rc::new(CachedPrimaryPlanAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryPlanAssignment>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary plan assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanAssignmentQuery {
    pub namespace: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryPlanAssignmentQuery {
    pub fn matches(&self, item: &PrimaryPlanAssignment) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[PrimaryPlanAssignment]) -> Vec<PrimaryPlanAssignment> {
        let matched = collections::filter(items, |candidate: &PrimaryPlanAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_plan_assignments(
    store: Rc<dyn PrimaryPlanAssignmentStore>,
    query: &PrimaryPlanAssignmentQuery,
) -> Vec<PrimaryPlanAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryPlanAssignmentStore> = new_memory_primary_plan_assignment_store();
        let item = new_primary_plan_assignment("code-383", "label-985");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
