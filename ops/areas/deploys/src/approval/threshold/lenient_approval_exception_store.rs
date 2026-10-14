use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient approval exceptions by code.
pub trait LenientApprovalExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientApprovalException>;
    fn save(&self, item: &LenientApprovalException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientApprovalException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientApprovalExceptionStore {
    pub items: RefCell<BTreeMap<String, LenientApprovalException>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientApprovalExceptionStore for MemoryLenientApprovalExceptionStore {
    fn find(&self, code: &str) -> Option<LenientApprovalException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientApprovalException) {
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

    fn list(&self) -> Vec<LenientApprovalException> {
        let mut out: Vec<LenientApprovalException> = Vec::new();
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

pub fn new_memory_lenient_approval_exception_store() -> Rc<dyn LenientApprovalExceptionStore> {
    Rc::new(MemoryLenientApprovalExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientApprovalExceptionStore {
    pub inner: Rc<dyn LenientApprovalExceptionStore>,
    pub cache: RefCell<cache::TtlCache<LenientApprovalException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientApprovalExceptionStore for CachedLenientApprovalExceptionStore {
    fn find(&self, code: &str) -> Option<LenientApprovalException> {
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

    fn save(&self, item: &LenientApprovalException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientApprovalException> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_approval_exception_store(
    inner: Rc<dyn LenientApprovalExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientApprovalExceptionStore> {
    Rc::new(CachedLenientApprovalExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientApprovalException>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient approval exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalExceptionQuery {
    pub label: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientApprovalExceptionQuery {
    pub fn matches(&self, item: &LenientApprovalException) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientApprovalException]) -> Vec<LenientApprovalException> {
        let matched = collections::filter(items, |candidate: &LenientApprovalException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_approval_exceptions(
    store: Rc<dyn LenientApprovalExceptionStore>,
    query: &LenientApprovalExceptionQuery,
) -> Vec<LenientApprovalException> {
    query.apply(&store.list())
}
