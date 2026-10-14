use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary audit event requests by code.
pub trait PrimaryAuditEventRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryAuditEventRequest>;
    fn save(&self, item: &PrimaryAuditEventRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryAuditEventRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryAuditEventRequestStore {
    pub items: RefCell<BTreeMap<String, PrimaryAuditEventRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryAuditEventRequestStore for MemoryPrimaryAuditEventRequestStore {
    fn find(&self, code: &str) -> Option<PrimaryAuditEventRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryAuditEventRequest) {
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

    fn list(&self) -> Vec<PrimaryAuditEventRequest> {
        let mut out: Vec<PrimaryAuditEventRequest> = Vec::new();
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

pub fn new_memory_primary_audit_event_request_store() -> Rc<dyn PrimaryAuditEventRequestStore> {
    Rc::new(MemoryPrimaryAuditEventRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryAuditEventRequestStore {
    pub inner: Rc<dyn PrimaryAuditEventRequestStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryAuditEventRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryAuditEventRequestStore for CachedPrimaryAuditEventRequestStore {
    fn find(&self, code: &str) -> Option<PrimaryAuditEventRequest> {
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

    fn save(&self, item: &PrimaryAuditEventRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryAuditEventRequest> {
        self.inner.list()
    }
}

pub fn new_cached_primary_audit_event_request_store(
    inner: Rc<dyn PrimaryAuditEventRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryAuditEventRequestStore> {
    Rc::new(CachedPrimaryAuditEventRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryAuditEventRequest>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary audit event requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAuditEventRequestQuery {
    pub name: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryAuditEventRequestQuery {
    pub fn matches(&self, item: &PrimaryAuditEventRequest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryAuditEventRequest]) -> Vec<PrimaryAuditEventRequest> {
        let matched = collections::filter(items, |candidate: &PrimaryAuditEventRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_audit_event_requests(
    store: Rc<dyn PrimaryAuditEventRequestStore>,
    query: &PrimaryAuditEventRequestQuery,
) -> Vec<PrimaryAuditEventRequest> {
    query.apply(&store.list())
}
