use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched rule assignments by code.
pub trait BatchedRuleAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedRuleAssignment>;
    fn save(&self, item: &BatchedRuleAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedRuleAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedRuleAssignmentStore {
    pub items: RefCell<BTreeMap<String, BatchedRuleAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedRuleAssignmentStore for MemoryBatchedRuleAssignmentStore {
    fn find(&self, code: &str) -> Option<BatchedRuleAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedRuleAssignment) {
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

    fn list(&self) -> Vec<BatchedRuleAssignment> {
        let mut out: Vec<BatchedRuleAssignment> = Vec::new();
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

pub fn new_memory_batched_rule_assignment_store() -> Rc<dyn BatchedRuleAssignmentStore> {
    Rc::new(MemoryBatchedRuleAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedRuleAssignmentStore {
    pub inner: Rc<dyn BatchedRuleAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<BatchedRuleAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedRuleAssignmentStore for CachedBatchedRuleAssignmentStore {
    fn find(&self, code: &str) -> Option<BatchedRuleAssignment> {
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

    fn save(&self, item: &BatchedRuleAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedRuleAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_batched_rule_assignment_store(
    inner: Rc<dyn BatchedRuleAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedRuleAssignmentStore> {
    Rc::new(CachedBatchedRuleAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedRuleAssignment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched rule assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAssignmentQuery {
    pub label: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedRuleAssignmentQuery {
    pub fn matches(&self, item: &BatchedRuleAssignment) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedRuleAssignment]) -> Vec<BatchedRuleAssignment> {
        let matched = collections::filter(items, |candidate: &BatchedRuleAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_rule_assignments(
    store: Rc<dyn BatchedRuleAssignmentStore>,
    query: &BatchedRuleAssignmentQuery,
) -> Vec<BatchedRuleAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedRuleAssignmentStore> = new_memory_batched_rule_assignment_store();
        let item = new_batched_rule_assignment("code-106", "label-971");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
