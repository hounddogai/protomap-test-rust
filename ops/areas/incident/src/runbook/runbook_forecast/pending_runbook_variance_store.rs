use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending runbook variances by code.
pub trait PendingRunbookVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingRunbookVariance>;
    fn save(&self, item: &PendingRunbookVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingRunbookVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingRunbookVarianceStore {
    pub items: RefCell<BTreeMap<String, PendingRunbookVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingRunbookVarianceStore for MemoryPendingRunbookVarianceStore {
    fn find(&self, code: &str) -> Option<PendingRunbookVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingRunbookVariance) {
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

    fn list(&self) -> Vec<PendingRunbookVariance> {
        let mut out: Vec<PendingRunbookVariance> = Vec::new();
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

pub fn new_memory_pending_runbook_variance_store() -> Rc<dyn PendingRunbookVarianceStore> {
    Rc::new(MemoryPendingRunbookVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingRunbookVarianceStore {
    pub inner: Rc<dyn PendingRunbookVarianceStore>,
    pub cache: RefCell<cache::TtlCache<PendingRunbookVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingRunbookVarianceStore for CachedPendingRunbookVarianceStore {
    fn find(&self, code: &str) -> Option<PendingRunbookVariance> {
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

    fn save(&self, item: &PendingRunbookVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingRunbookVariance> {
        self.inner.list()
    }
}

pub fn new_cached_pending_runbook_variance_store(
    inner: Rc<dyn PendingRunbookVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingRunbookVarianceStore> {
    Rc::new(CachedPendingRunbookVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingRunbookVariance>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending runbook variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookVarianceQuery {
    pub channel: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingRunbookVarianceQuery {
    pub fn matches(&self, item: &PendingRunbookVariance) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingRunbookVariance]) -> Vec<PendingRunbookVariance> {
        let matched = collections::filter(items, |candidate: &PendingRunbookVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_runbook_variances(
    store: Rc<dyn PendingRunbookVarianceStore>,
    query: &PendingRunbookVarianceQuery,
) -> Vec<PendingRunbookVariance> {
    query.apply(&store.list())
}
