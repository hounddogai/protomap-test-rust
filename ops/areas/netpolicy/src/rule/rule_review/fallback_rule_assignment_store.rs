use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback rule assignments by code.
pub trait FallbackRuleAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackRuleAssignment>;
    fn save(&self, item: &FallbackRuleAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackRuleAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackRuleAssignmentStore {
    pub items: RefCell<BTreeMap<String, FallbackRuleAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackRuleAssignmentStore for MemoryFallbackRuleAssignmentStore {
    fn find(&self, code: &str) -> Option<FallbackRuleAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackRuleAssignment) {
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

    fn list(&self) -> Vec<FallbackRuleAssignment> {
        let mut out: Vec<FallbackRuleAssignment> = Vec::new();
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

pub fn new_memory_fallback_rule_assignment_store() -> Rc<dyn FallbackRuleAssignmentStore> {
    Rc::new(MemoryFallbackRuleAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackRuleAssignmentStore {
    pub inner: Rc<dyn FallbackRuleAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<FallbackRuleAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackRuleAssignmentStore for CachedFallbackRuleAssignmentStore {
    fn find(&self, code: &str) -> Option<FallbackRuleAssignment> {
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

    fn save(&self, item: &FallbackRuleAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackRuleAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_rule_assignment_store(
    inner: Rc<dyn FallbackRuleAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackRuleAssignmentStore> {
    Rc::new(CachedFallbackRuleAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackRuleAssignment>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback rule assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleAssignmentQuery {
    pub title: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackRuleAssignmentQuery {
    pub fn matches(&self, item: &FallbackRuleAssignment) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackRuleAssignment]) -> Vec<FallbackRuleAssignment> {
        let matched = collections::filter(items, |candidate: &FallbackRuleAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_rule_assignments(
    store: Rc<dyn FallbackRuleAssignmentStore>,
    query: &FallbackRuleAssignmentQuery,
) -> Vec<FallbackRuleAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackRuleAssignmentStore> = new_memory_fallback_rule_assignment_store();
        let item = new_fallback_rule_assignment("code-51", "title-13");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
