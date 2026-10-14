use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling isolation audits by code.
pub trait RollingIsolationAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingIsolationAudit>;
    fn save(&self, item: &RollingIsolationAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingIsolationAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingIsolationAuditStore {
    pub items: RefCell<BTreeMap<String, RollingIsolationAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingIsolationAuditStore for MemoryRollingIsolationAuditStore {
    fn find(&self, code: &str) -> Option<RollingIsolationAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingIsolationAudit) {
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

    fn list(&self) -> Vec<RollingIsolationAudit> {
        let mut out: Vec<RollingIsolationAudit> = Vec::new();
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

pub fn new_memory_rolling_isolation_audit_store() -> Rc<dyn RollingIsolationAuditStore> {
    Rc::new(MemoryRollingIsolationAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingIsolationAuditStore {
    pub inner: Rc<dyn RollingIsolationAuditStore>,
    pub cache: RefCell<cache::TtlCache<RollingIsolationAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingIsolationAuditStore for CachedRollingIsolationAuditStore {
    fn find(&self, code: &str) -> Option<RollingIsolationAudit> {
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

    fn save(&self, item: &RollingIsolationAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingIsolationAudit> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_isolation_audit_store(
    inner: Rc<dyn RollingIsolationAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingIsolationAuditStore> {
    Rc::new(CachedRollingIsolationAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingIsolationAudit>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling isolation audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationAuditQuery {
    pub label: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingIsolationAuditQuery {
    pub fn matches(&self, item: &RollingIsolationAudit) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingIsolationAudit]) -> Vec<RollingIsolationAudit> {
        let matched = collections::filter(items, |candidate: &RollingIsolationAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_isolation_audits(
    store: Rc<dyn RollingIsolationAuditStore>,
    query: &RollingIsolationAuditQuery,
) -> Vec<RollingIsolationAudit> {
    query.apply(&store.list())
}
