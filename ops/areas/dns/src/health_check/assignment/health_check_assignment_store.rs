use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps health check assignments by code.
pub trait HealthCheckAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<HealthCheckAssignment>;
    fn save(&self, item: &HealthCheckAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<HealthCheckAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHealthCheckAssignmentStore {
    pub items: RefCell<BTreeMap<String, HealthCheckAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl HealthCheckAssignmentStore for MemoryHealthCheckAssignmentStore {
    fn find(&self, code: &str) -> Option<HealthCheckAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &HealthCheckAssignment) {
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

    fn list(&self) -> Vec<HealthCheckAssignment> {
        let mut out: Vec<HealthCheckAssignment> = Vec::new();
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

pub fn new_memory_health_check_assignment_store() -> Rc<dyn HealthCheckAssignmentStore> {
    Rc::new(MemoryHealthCheckAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedHealthCheckAssignmentStore {
    pub inner: Rc<dyn HealthCheckAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<HealthCheckAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl HealthCheckAssignmentStore for CachedHealthCheckAssignmentStore {
    fn find(&self, code: &str) -> Option<HealthCheckAssignment> {
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

    fn save(&self, item: &HealthCheckAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<HealthCheckAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_health_check_assignment_store(
    inner: Rc<dyn HealthCheckAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn HealthCheckAssignmentStore> {
    Rc::new(CachedHealthCheckAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<HealthCheckAssignment>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects health check assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAssignmentQuery {
    pub cluster: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl HealthCheckAssignmentQuery {
    pub fn matches(&self, item: &HealthCheckAssignment) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[HealthCheckAssignment]) -> Vec<HealthCheckAssignment> {
        let matched = collections::filter(items, |candidate: &HealthCheckAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_health_check_assignments(
    store: Rc<dyn HealthCheckAssignmentStore>,
    query: &HealthCheckAssignmentQuery,
) -> Vec<HealthCheckAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn HealthCheckAssignmentStore> = new_memory_health_check_assignment_store();
        let item = new_health_check_assignment("code-229", "label-274");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
