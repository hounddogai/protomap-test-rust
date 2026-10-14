use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped dashboard exceptions by code.
pub trait CappedDashboardExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedDashboardException>;
    fn save(&self, item: &CappedDashboardException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedDashboardException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedDashboardExceptionStore {
    pub items: RefCell<BTreeMap<String, CappedDashboardException>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedDashboardExceptionStore for MemoryCappedDashboardExceptionStore {
    fn find(&self, code: &str) -> Option<CappedDashboardException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedDashboardException) {
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

    fn list(&self) -> Vec<CappedDashboardException> {
        let mut out: Vec<CappedDashboardException> = Vec::new();
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

pub fn new_memory_capped_dashboard_exception_store() -> Rc<dyn CappedDashboardExceptionStore> {
    Rc::new(MemoryCappedDashboardExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedDashboardExceptionStore {
    pub inner: Rc<dyn CappedDashboardExceptionStore>,
    pub cache: RefCell<cache::TtlCache<CappedDashboardException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedDashboardExceptionStore for CachedCappedDashboardExceptionStore {
    fn find(&self, code: &str) -> Option<CappedDashboardException> {
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

    fn save(&self, item: &CappedDashboardException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedDashboardException> {
        self.inner.list()
    }
}

pub fn new_cached_capped_dashboard_exception_store(
    inner: Rc<dyn CappedDashboardExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedDashboardExceptionStore> {
    Rc::new(CachedCappedDashboardExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedDashboardException>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped dashboard exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardExceptionQuery {
    pub country: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedDashboardExceptionQuery {
    pub fn matches(&self, item: &CappedDashboardException) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedDashboardException]) -> Vec<CappedDashboardException> {
        let matched = collections::filter(items, |candidate: &CappedDashboardException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_dashboard_exceptions(
    store: Rc<dyn CappedDashboardExceptionStore>,
    query: &CappedDashboardExceptionQuery,
) -> Vec<CappedDashboardException> {
    query.apply(&store.list())
}
