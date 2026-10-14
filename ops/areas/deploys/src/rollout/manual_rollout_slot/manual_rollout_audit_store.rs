use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual rollout audits by code.
pub trait ManualRolloutAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualRolloutAudit>;
    fn save(&self, item: &ManualRolloutAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualRolloutAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualRolloutAuditStore {
    pub items: RefCell<BTreeMap<String, ManualRolloutAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualRolloutAuditStore for MemoryManualRolloutAuditStore {
    fn find(&self, code: &str) -> Option<ManualRolloutAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualRolloutAudit) {
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

    fn list(&self) -> Vec<ManualRolloutAudit> {
        let mut out: Vec<ManualRolloutAudit> = Vec::new();
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

pub fn new_memory_manual_rollout_audit_store() -> Rc<dyn ManualRolloutAuditStore> {
    Rc::new(MemoryManualRolloutAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualRolloutAuditStore {
    pub inner: Rc<dyn ManualRolloutAuditStore>,
    pub cache: RefCell<cache::TtlCache<ManualRolloutAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualRolloutAuditStore for CachedManualRolloutAuditStore {
    fn find(&self, code: &str) -> Option<ManualRolloutAudit> {
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

    fn save(&self, item: &ManualRolloutAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualRolloutAudit> {
        self.inner.list()
    }
}

pub fn new_cached_manual_rollout_audit_store(
    inner: Rc<dyn ManualRolloutAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualRolloutAuditStore> {
    Rc::new(CachedManualRolloutAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualRolloutAudit>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual rollout audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRolloutAuditQuery {
    pub label: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualRolloutAuditQuery {
    pub fn matches(&self, item: &ManualRolloutAudit) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[ManualRolloutAudit]) -> Vec<ManualRolloutAudit> {
        let matched = collections::filter(items, |candidate: &ManualRolloutAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_rollout_audits(
    store: Rc<dyn ManualRolloutAuditStore>,
    query: &ManualRolloutAuditQuery,
) -> Vec<ManualRolloutAudit> {
    query.apply(&store.list())
}
