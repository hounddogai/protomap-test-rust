use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling approval summaries by code.
pub trait RollingApprovalSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingApprovalSummary>;
    fn save(&self, item: &RollingApprovalSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingApprovalSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingApprovalSummaryStore {
    pub items: RefCell<BTreeMap<String, RollingApprovalSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingApprovalSummaryStore for MemoryRollingApprovalSummaryStore {
    fn find(&self, code: &str) -> Option<RollingApprovalSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingApprovalSummary) {
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

    fn list(&self) -> Vec<RollingApprovalSummary> {
        let mut out: Vec<RollingApprovalSummary> = Vec::new();
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

pub fn new_memory_rolling_approval_summary_store() -> Rc<dyn RollingApprovalSummaryStore> {
    Rc::new(MemoryRollingApprovalSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingApprovalSummaryStore {
    pub inner: Rc<dyn RollingApprovalSummaryStore>,
    pub cache: RefCell<cache::TtlCache<RollingApprovalSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingApprovalSummaryStore for CachedRollingApprovalSummaryStore {
    fn find(&self, code: &str) -> Option<RollingApprovalSummary> {
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

    fn save(&self, item: &RollingApprovalSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingApprovalSummary> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_approval_summary_store(
    inner: Rc<dyn RollingApprovalSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingApprovalSummaryStore> {
    Rc::new(CachedRollingApprovalSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingApprovalSummary>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling approval summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalSummaryQuery {
    pub unit: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingApprovalSummaryQuery {
    pub fn matches(&self, item: &RollingApprovalSummary) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingApprovalSummary]) -> Vec<RollingApprovalSummary> {
        let matched = collections::filter(items, |candidate: &RollingApprovalSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_approval_summaries(
    store: Rc<dyn RollingApprovalSummaryStore>,
    query: &RollingApprovalSummaryQuery,
) -> Vec<RollingApprovalSummary> {
    query.apply(&store.list())
}
