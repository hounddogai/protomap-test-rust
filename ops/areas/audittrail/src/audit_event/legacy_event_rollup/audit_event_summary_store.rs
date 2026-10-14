use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event summaries by code.
pub trait AuditEventSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventSummary>;
    fn save(&self, item: &AuditEventSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventSummaryStore {
    pub items: RefCell<BTreeMap<String, AuditEventSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventSummaryStore for MemoryAuditEventSummaryStore {
    fn find(&self, code: &str) -> Option<AuditEventSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventSummary) {
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

    fn list(&self) -> Vec<AuditEventSummary> {
        let mut out: Vec<AuditEventSummary> = Vec::new();
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

pub fn new_memory_audit_event_summary_store() -> Rc<dyn AuditEventSummaryStore> {
    Rc::new(MemoryAuditEventSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventSummaryStore {
    pub inner: Rc<dyn AuditEventSummaryStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventSummaryStore for CachedAuditEventSummaryStore {
    fn find(&self, code: &str) -> Option<AuditEventSummary> {
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

    fn save(&self, item: &AuditEventSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventSummary> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_summary_store(
    inner: Rc<dyn AuditEventSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventSummaryStore> {
    Rc::new(CachedAuditEventSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventSummary>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummaryQuery {
    pub name: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventSummaryQuery {
    pub fn matches(&self, item: &AuditEventSummary) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AuditEventSummary]) -> Vec<AuditEventSummary> {
        let matched = collections::filter(items, |candidate: &AuditEventSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_summaries(
    store: Rc<dyn AuditEventSummaryStore>,
    query: &AuditEventSummaryQuery,
) -> Vec<AuditEventSummary> {
    query.apply(&store.list())
}
