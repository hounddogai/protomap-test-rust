use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary peering audits by code.
pub trait PrimaryPeeringAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryPeeringAudit>;
    fn save(&self, item: &PrimaryPeeringAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryPeeringAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryPeeringAuditStore {
    pub items: RefCell<BTreeMap<String, PrimaryPeeringAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryPeeringAuditStore for MemoryPrimaryPeeringAuditStore {
    fn find(&self, code: &str) -> Option<PrimaryPeeringAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryPeeringAudit) {
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

    fn list(&self) -> Vec<PrimaryPeeringAudit> {
        let mut out: Vec<PrimaryPeeringAudit> = Vec::new();
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

pub fn new_memory_primary_peering_audit_store() -> Rc<dyn PrimaryPeeringAuditStore> {
    Rc::new(MemoryPrimaryPeeringAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryPeeringAuditStore {
    pub inner: Rc<dyn PrimaryPeeringAuditStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryPeeringAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryPeeringAuditStore for CachedPrimaryPeeringAuditStore {
    fn find(&self, code: &str) -> Option<PrimaryPeeringAudit> {
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

    fn save(&self, item: &PrimaryPeeringAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryPeeringAudit> {
        self.inner.list()
    }
}

pub fn new_cached_primary_peering_audit_store(
    inner: Rc<dyn PrimaryPeeringAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryPeeringAuditStore> {
    Rc::new(CachedPrimaryPeeringAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryPeeringAudit>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary peering audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringAuditQuery {
    pub channel: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryPeeringAuditQuery {
    pub fn matches(&self, item: &PrimaryPeeringAudit) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryPeeringAudit]) -> Vec<PrimaryPeeringAudit> {
        let matched = collections::filter(items, |candidate: &PrimaryPeeringAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_peering_audits(
    store: Rc<dyn PrimaryPeeringAuditStore>,
    query: &PrimaryPeeringAuditQuery,
) -> Vec<PrimaryPeeringAudit> {
    query.apply(&store.list())
}
