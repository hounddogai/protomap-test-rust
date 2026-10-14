use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending audit event slots by code.
pub trait PendingAuditEventSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingAuditEventSlot>;
    fn save(&self, item: &PendingAuditEventSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingAuditEventSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingAuditEventSlotStore {
    pub items: RefCell<BTreeMap<String, PendingAuditEventSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingAuditEventSlotStore for MemoryPendingAuditEventSlotStore {
    fn find(&self, code: &str) -> Option<PendingAuditEventSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingAuditEventSlot) {
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

    fn list(&self) -> Vec<PendingAuditEventSlot> {
        let mut out: Vec<PendingAuditEventSlot> = Vec::new();
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

pub fn new_memory_pending_audit_event_slot_store() -> Rc<dyn PendingAuditEventSlotStore> {
    Rc::new(MemoryPendingAuditEventSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingAuditEventSlotStore {
    pub inner: Rc<dyn PendingAuditEventSlotStore>,
    pub cache: RefCell<cache::TtlCache<PendingAuditEventSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingAuditEventSlotStore for CachedPendingAuditEventSlotStore {
    fn find(&self, code: &str) -> Option<PendingAuditEventSlot> {
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

    fn save(&self, item: &PendingAuditEventSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingAuditEventSlot> {
        self.inner.list()
    }
}

pub fn new_cached_pending_audit_event_slot_store(
    inner: Rc<dyn PendingAuditEventSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingAuditEventSlotStore> {
    Rc::new(CachedPendingAuditEventSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingAuditEventSlot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending audit event slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventSlotQuery {
    pub title: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingAuditEventSlotQuery {
    pub fn matches(&self, item: &PendingAuditEventSlot) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingAuditEventSlot]) -> Vec<PendingAuditEventSlot> {
        let matched = collections::filter(items, |candidate: &PendingAuditEventSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_audit_event_slots(
    store: Rc<dyn PendingAuditEventSlotStore>,
    query: &PendingAuditEventSlotQuery,
) -> Vec<PendingAuditEventSlot> {
    query.apply(&store.list())
}
