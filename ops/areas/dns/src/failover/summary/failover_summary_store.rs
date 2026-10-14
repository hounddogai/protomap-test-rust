use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps failover summaries by code.
pub trait FailoverSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FailoverSummary>;
    fn save(&self, item: &FailoverSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FailoverSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFailoverSummaryStore {
    pub items: RefCell<BTreeMap<String, FailoverSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl FailoverSummaryStore for MemoryFailoverSummaryStore {
    fn find(&self, code: &str) -> Option<FailoverSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FailoverSummary) {
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

    fn list(&self) -> Vec<FailoverSummary> {
        let mut out: Vec<FailoverSummary> = Vec::new();
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

pub fn new_memory_failover_summary_store() -> Rc<dyn FailoverSummaryStore> {
    Rc::new(MemoryFailoverSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFailoverSummaryStore {
    pub inner: Rc<dyn FailoverSummaryStore>,
    pub cache: RefCell<cache::TtlCache<FailoverSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FailoverSummaryStore for CachedFailoverSummaryStore {
    fn find(&self, code: &str) -> Option<FailoverSummary> {
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

    fn save(&self, item: &FailoverSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FailoverSummary> {
        self.inner.list()
    }
}

pub fn new_cached_failover_summary_store(
    inner: Rc<dyn FailoverSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FailoverSummaryStore> {
    Rc::new(CachedFailoverSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FailoverSummary>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects failover summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSummaryQuery {
    pub label: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FailoverSummaryQuery {
    pub fn matches(&self, item: &FailoverSummary) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FailoverSummary]) -> Vec<FailoverSummary> {
        let matched = collections::filter(items, |candidate: &FailoverSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_failover_summaries(
    store: Rc<dyn FailoverSummaryStore>,
    query: &FailoverSummaryQuery,
) -> Vec<FailoverSummary> {
    query.apply(&store.list())
}
