use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event profiles by code.
pub trait AuditEventProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventProfile>;
    fn save(&self, item: &AuditEventProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventProfileStore {
    pub items: RefCell<BTreeMap<String, AuditEventProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventProfileStore for MemoryAuditEventProfileStore {
    fn find(&self, code: &str) -> Option<AuditEventProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventProfile) {
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

    fn list(&self) -> Vec<AuditEventProfile> {
        let mut out: Vec<AuditEventProfile> = Vec::new();
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

pub fn new_memory_audit_event_profile_store() -> Rc<dyn AuditEventProfileStore> {
    Rc::new(MemoryAuditEventProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventProfileStore {
    pub inner: Rc<dyn AuditEventProfileStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventProfileStore for CachedAuditEventProfileStore {
    fn find(&self, code: &str) -> Option<AuditEventProfile> {
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

    fn save(&self, item: &AuditEventProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventProfile> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_profile_store(
    inner: Rc<dyn AuditEventProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventProfileStore> {
    Rc::new(CachedAuditEventProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventProfile>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventProfileQuery {
    pub title: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventProfileQuery {
    pub fn matches(&self, item: &AuditEventProfile) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[AuditEventProfile]) -> Vec<AuditEventProfile> {
        let matched = collections::filter(items, |candidate: &AuditEventProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_profiles(
    store: Rc<dyn AuditEventProfileStore>,
    query: &AuditEventProfileQuery,
) -> Vec<AuditEventProfile> {
    query.apply(&store.list())
}
