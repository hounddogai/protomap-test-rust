use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global evidence exceptions by code.
pub trait GlobalEvidenceExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalEvidenceException>;
    fn save(&self, item: &GlobalEvidenceException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalEvidenceException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalEvidenceExceptionStore {
    pub items: RefCell<BTreeMap<String, GlobalEvidenceException>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalEvidenceExceptionStore for MemoryGlobalEvidenceExceptionStore {
    fn find(&self, code: &str) -> Option<GlobalEvidenceException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalEvidenceException) {
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

    fn list(&self) -> Vec<GlobalEvidenceException> {
        let mut out: Vec<GlobalEvidenceException> = Vec::new();
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

pub fn new_memory_global_evidence_exception_store() -> Rc<dyn GlobalEvidenceExceptionStore> {
    Rc::new(MemoryGlobalEvidenceExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalEvidenceExceptionStore {
    pub inner: Rc<dyn GlobalEvidenceExceptionStore>,
    pub cache: RefCell<cache::TtlCache<GlobalEvidenceException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalEvidenceExceptionStore for CachedGlobalEvidenceExceptionStore {
    fn find(&self, code: &str) -> Option<GlobalEvidenceException> {
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

    fn save(&self, item: &GlobalEvidenceException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalEvidenceException> {
        self.inner.list()
    }
}

pub fn new_cached_global_evidence_exception_store(
    inner: Rc<dyn GlobalEvidenceExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalEvidenceExceptionStore> {
    Rc::new(CachedGlobalEvidenceExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalEvidenceException>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global evidence exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceExceptionQuery {
    pub title: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalEvidenceExceptionQuery {
    pub fn matches(&self, item: &GlobalEvidenceException) -> bool {
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

    pub fn apply(&self, items: &[GlobalEvidenceException]) -> Vec<GlobalEvidenceException> {
        let matched = collections::filter(items, |candidate: &GlobalEvidenceException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_evidence_exceptions(
    store: Rc<dyn GlobalEvidenceExceptionStore>,
    query: &GlobalEvidenceExceptionQuery,
) -> Vec<GlobalEvidenceException> {
    query.apply(&store.list())
}
