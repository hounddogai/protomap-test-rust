use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled redaction assignments by code.
pub trait ScheduledRedactionAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledRedactionAssignment>;
    fn save(&self, item: &ScheduledRedactionAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledRedactionAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledRedactionAssignmentStore {
    pub items: RefCell<BTreeMap<String, ScheduledRedactionAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledRedactionAssignmentStore for MemoryScheduledRedactionAssignmentStore {
    fn find(&self, code: &str) -> Option<ScheduledRedactionAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledRedactionAssignment) {
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

    fn list(&self) -> Vec<ScheduledRedactionAssignment> {
        let mut out: Vec<ScheduledRedactionAssignment> = Vec::new();
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

pub fn new_memory_scheduled_redaction_assignment_store() -> Rc<dyn ScheduledRedactionAssignmentStore> {
    Rc::new(MemoryScheduledRedactionAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledRedactionAssignmentStore {
    pub inner: Rc<dyn ScheduledRedactionAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledRedactionAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledRedactionAssignmentStore for CachedScheduledRedactionAssignmentStore {
    fn find(&self, code: &str) -> Option<ScheduledRedactionAssignment> {
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

    fn save(&self, item: &ScheduledRedactionAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledRedactionAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_redaction_assignment_store(
    inner: Rc<dyn ScheduledRedactionAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledRedactionAssignmentStore> {
    Rc::new(CachedScheduledRedactionAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledRedactionAssignment>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled redaction assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRedactionAssignmentQuery {
    pub name: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledRedactionAssignmentQuery {
    pub fn matches(&self, item: &ScheduledRedactionAssignment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledRedactionAssignment]) -> Vec<ScheduledRedactionAssignment> {
        let matched = collections::filter(items, |candidate: &ScheduledRedactionAssignment| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_redaction_assignments(
    store: Rc<dyn ScheduledRedactionAssignmentStore>,
    query: &ScheduledRedactionAssignmentQuery,
) -> Vec<ScheduledRedactionAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledRedactionAssignmentStore> = new_memory_scheduled_redaction_assignment_store();
        let item = new_scheduled_redaction_assignment("code-639", "name-595");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
