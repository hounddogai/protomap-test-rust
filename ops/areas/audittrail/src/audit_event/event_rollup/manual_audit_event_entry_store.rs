use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual audit event entries by code.
pub trait ManualAuditEventEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualAuditEventEntry>;
    fn save(&self, item: &ManualAuditEventEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualAuditEventEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualAuditEventEntryStore {
    pub items: RefCell<BTreeMap<String, ManualAuditEventEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualAuditEventEntryStore for MemoryManualAuditEventEntryStore {
    fn find(&self, code: &str) -> Option<ManualAuditEventEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualAuditEventEntry) {
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

    fn list(&self) -> Vec<ManualAuditEventEntry> {
        let mut out: Vec<ManualAuditEventEntry> = Vec::new();
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

pub fn new_memory_manual_audit_event_entry_store() -> Rc<dyn ManualAuditEventEntryStore> {
    Rc::new(MemoryManualAuditEventEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualAuditEventEntryStore {
    pub inner: Rc<dyn ManualAuditEventEntryStore>,
    pub cache: RefCell<cache::TtlCache<ManualAuditEventEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualAuditEventEntryStore for CachedManualAuditEventEntryStore {
    fn find(&self, code: &str) -> Option<ManualAuditEventEntry> {
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

    fn save(&self, item: &ManualAuditEventEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualAuditEventEntry> {
        self.inner.list()
    }
}

pub fn new_cached_manual_audit_event_entry_store(
    inner: Rc<dyn ManualAuditEventEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualAuditEventEntryStore> {
    Rc::new(CachedManualAuditEventEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualAuditEventEntry>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual audit event entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventEntryQuery {
    pub unit: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualAuditEventEntryQuery {
    pub fn matches(&self, item: &ManualAuditEventEntry) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[ManualAuditEventEntry]) -> Vec<ManualAuditEventEntry> {
        let matched = collections::filter(items, |candidate: &ManualAuditEventEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_audit_event_entries(
    store: Rc<dyn ManualAuditEventEntryStore>,
    query: &ManualAuditEventEntryQuery,
) -> Vec<ManualAuditEventEntry> {
    query.apply(&store.list())
}
