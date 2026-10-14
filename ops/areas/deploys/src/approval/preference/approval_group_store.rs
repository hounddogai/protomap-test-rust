use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval groups by code.
pub trait ApprovalGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalGroup>;
    fn save(&self, item: &ApprovalGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalGroupStore {
    pub items: RefCell<BTreeMap<String, ApprovalGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalGroupStore for MemoryApprovalGroupStore {
    fn find(&self, code: &str) -> Option<ApprovalGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalGroup) {
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

    fn list(&self) -> Vec<ApprovalGroup> {
        let mut out: Vec<ApprovalGroup> = Vec::new();
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

pub fn new_memory_approval_group_store() -> Rc<dyn ApprovalGroupStore> {
    Rc::new(MemoryApprovalGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalGroupStore {
    pub inner: Rc<dyn ApprovalGroupStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalGroupStore for CachedApprovalGroupStore {
    fn find(&self, code: &str) -> Option<ApprovalGroup> {
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

    fn save(&self, item: &ApprovalGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalGroup> {
        self.inner.list()
    }
}

pub fn new_cached_approval_group_store(
    inner: Rc<dyn ApprovalGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalGroupStore> {
    Rc::new(CachedApprovalGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalGroup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalGroupQuery {
    pub label: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalGroupQuery {
    pub fn matches(&self, item: &ApprovalGroup) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[ApprovalGroup]) -> Vec<ApprovalGroup> {
        let matched = collections::filter(items, |candidate: &ApprovalGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_groups(store: Rc<dyn ApprovalGroupStore>, query: &ApprovalGroupQuery) -> Vec<ApprovalGroup> {
    query.apply(&store.list())
}
