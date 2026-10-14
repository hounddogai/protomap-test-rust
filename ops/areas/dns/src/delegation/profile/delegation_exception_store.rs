use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps delegation exceptions by code.
pub trait DelegationExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DelegationException>;
    fn save(&self, item: &DelegationException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DelegationException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelegationExceptionStore {
    pub items: RefCell<BTreeMap<String, DelegationException>>,
    pub order: RefCell<Vec<String>>,
}

impl DelegationExceptionStore for MemoryDelegationExceptionStore {
    fn find(&self, code: &str) -> Option<DelegationException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DelegationException) {
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

    fn list(&self) -> Vec<DelegationException> {
        let mut out: Vec<DelegationException> = Vec::new();
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

pub fn new_memory_delegation_exception_store() -> Rc<dyn DelegationExceptionStore> {
    Rc::new(MemoryDelegationExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDelegationExceptionStore {
    pub inner: Rc<dyn DelegationExceptionStore>,
    pub cache: RefCell<cache::TtlCache<DelegationException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DelegationExceptionStore for CachedDelegationExceptionStore {
    fn find(&self, code: &str) -> Option<DelegationException> {
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

    fn save(&self, item: &DelegationException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DelegationException> {
        self.inner.list()
    }
}

pub fn new_cached_delegation_exception_store(
    inner: Rc<dyn DelegationExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DelegationExceptionStore> {
    Rc::new(CachedDelegationExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DelegationException>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects delegation exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationExceptionQuery {
    pub segment: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DelegationExceptionQuery {
    pub fn matches(&self, item: &DelegationException) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[DelegationException]) -> Vec<DelegationException> {
        let matched = collections::filter(items, |candidate: &DelegationException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_delegation_exceptions(
    store: Rc<dyn DelegationExceptionStore>,
    query: &DelegationExceptionQuery,
) -> Vec<DelegationException> {
    query.apply(&store.list())
}
