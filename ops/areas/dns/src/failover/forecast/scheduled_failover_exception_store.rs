use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled failover exceptions by code.
pub trait ScheduledFailoverExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledFailoverException>;
    fn save(&self, item: &ScheduledFailoverException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledFailoverException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledFailoverExceptionStore {
    pub items: RefCell<BTreeMap<String, ScheduledFailoverException>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledFailoverExceptionStore for MemoryScheduledFailoverExceptionStore {
    fn find(&self, code: &str) -> Option<ScheduledFailoverException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledFailoverException) {
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

    fn list(&self) -> Vec<ScheduledFailoverException> {
        let mut out: Vec<ScheduledFailoverException> = Vec::new();
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

pub fn new_memory_scheduled_failover_exception_store() -> Rc<dyn ScheduledFailoverExceptionStore> {
    Rc::new(MemoryScheduledFailoverExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledFailoverExceptionStore {
    pub inner: Rc<dyn ScheduledFailoverExceptionStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledFailoverException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledFailoverExceptionStore for CachedScheduledFailoverExceptionStore {
    fn find(&self, code: &str) -> Option<ScheduledFailoverException> {
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

    fn save(&self, item: &ScheduledFailoverException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledFailoverException> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_failover_exception_store(
    inner: Rc<dyn ScheduledFailoverExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledFailoverExceptionStore> {
    Rc::new(CachedScheduledFailoverExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledFailoverException>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled failover exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverExceptionQuery {
    pub label: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledFailoverExceptionQuery {
    pub fn matches(&self, item: &ScheduledFailoverException) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledFailoverException]) -> Vec<ScheduledFailoverException> {
        let matched = collections::filter(items, |candidate: &ScheduledFailoverException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_failover_exceptions(
    store: Rc<dyn ScheduledFailoverExceptionStore>,
    query: &ScheduledFailoverExceptionQuery,
) -> Vec<ScheduledFailoverException> {
    query.apply(&store.list())
}
