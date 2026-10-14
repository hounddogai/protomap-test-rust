use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary credential batches by code.
pub trait PrimaryCredentialBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryCredentialBatch>;
    fn save(&self, item: &PrimaryCredentialBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryCredentialBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryCredentialBatchStore {
    pub items: RefCell<BTreeMap<String, PrimaryCredentialBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryCredentialBatchStore for MemoryPrimaryCredentialBatchStore {
    fn find(&self, code: &str) -> Option<PrimaryCredentialBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryCredentialBatch) {
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

    fn list(&self) -> Vec<PrimaryCredentialBatch> {
        let mut out: Vec<PrimaryCredentialBatch> = Vec::new();
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

pub fn new_memory_primary_credential_batch_store() -> Rc<dyn PrimaryCredentialBatchStore> {
    Rc::new(MemoryPrimaryCredentialBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryCredentialBatchStore {
    pub inner: Rc<dyn PrimaryCredentialBatchStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryCredentialBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryCredentialBatchStore for CachedPrimaryCredentialBatchStore {
    fn find(&self, code: &str) -> Option<PrimaryCredentialBatch> {
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

    fn save(&self, item: &PrimaryCredentialBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryCredentialBatch> {
        self.inner.list()
    }
}

pub fn new_cached_primary_credential_batch_store(
    inner: Rc<dyn PrimaryCredentialBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryCredentialBatchStore> {
    Rc::new(CachedPrimaryCredentialBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryCredentialBatch>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary credential batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCredentialBatchQuery {
    pub label: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryCredentialBatchQuery {
    pub fn matches(&self, item: &PrimaryCredentialBatch) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryCredentialBatch]) -> Vec<PrimaryCredentialBatch> {
        let matched = collections::filter(items, |candidate: &PrimaryCredentialBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_credential_batches(
    store: Rc<dyn PrimaryCredentialBatchStore>,
    query: &PrimaryCredentialBatchQuery,
) -> Vec<PrimaryCredentialBatch> {
    query.apply(&store.list())
}
