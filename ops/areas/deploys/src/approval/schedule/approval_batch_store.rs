use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval batches by code.
pub trait ApprovalBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalBatch>;
    fn save(&self, item: &ApprovalBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalBatchStore {
    pub items: RefCell<BTreeMap<String, ApprovalBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalBatchStore for MemoryApprovalBatchStore {
    fn find(&self, code: &str) -> Option<ApprovalBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalBatch) {
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

    fn list(&self) -> Vec<ApprovalBatch> {
        let mut out: Vec<ApprovalBatch> = Vec::new();
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

pub fn new_memory_approval_batch_store() -> Rc<dyn ApprovalBatchStore> {
    Rc::new(MemoryApprovalBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalBatchStore {
    pub inner: Rc<dyn ApprovalBatchStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalBatchStore for CachedApprovalBatchStore {
    fn find(&self, code: &str) -> Option<ApprovalBatch> {
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

    fn save(&self, item: &ApprovalBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalBatch> {
        self.inner.list()
    }
}

pub fn new_cached_approval_batch_store(
    inner: Rc<dyn ApprovalBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalBatchStore> {
    Rc::new(CachedApprovalBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalBatch>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBatchQuery {
    pub category: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalBatchQuery {
    pub fn matches(&self, item: &ApprovalBatch) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[ApprovalBatch]) -> Vec<ApprovalBatch> {
        let matched = collections::filter(items, |candidate: &ApprovalBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_batches(store: Rc<dyn ApprovalBatchStore>, query: &ApprovalBatchQuery) -> Vec<ApprovalBatch> {
    query.apply(&store.list())
}
