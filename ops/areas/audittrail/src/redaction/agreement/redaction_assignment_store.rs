use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction assignments by code.
pub trait RedactionAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionAssignment>;
    fn save(&self, item: &RedactionAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionAssignmentStore {
    pub items: RefCell<BTreeMap<String, RedactionAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionAssignmentStore for MemoryRedactionAssignmentStore {
    fn find(&self, code: &str) -> Option<RedactionAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionAssignment) {
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

    fn list(&self) -> Vec<RedactionAssignment> {
        let mut out: Vec<RedactionAssignment> = Vec::new();
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

pub fn new_memory_redaction_assignment_store() -> Rc<dyn RedactionAssignmentStore> {
    Rc::new(MemoryRedactionAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionAssignmentStore {
    pub inner: Rc<dyn RedactionAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<RedactionAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionAssignmentStore for CachedRedactionAssignmentStore {
    fn find(&self, code: &str) -> Option<RedactionAssignment> {
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

    fn save(&self, item: &RedactionAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_assignment_store(
    inner: Rc<dyn RedactionAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionAssignmentStore> {
    Rc::new(CachedRedactionAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionAssignment>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAssignmentQuery {
    pub cluster: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionAssignmentQuery {
    pub fn matches(&self, item: &RedactionAssignment) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RedactionAssignment]) -> Vec<RedactionAssignment> {
        let matched = collections::filter(items, |candidate: &RedactionAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_assignments(
    store: Rc<dyn RedactionAssignmentStore>,
    query: &RedactionAssignmentQuery,
) -> Vec<RedactionAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RedactionAssignmentStore> = new_memory_redaction_assignment_store();
        let item = new_redaction_assignment("code-166", "title-50");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
