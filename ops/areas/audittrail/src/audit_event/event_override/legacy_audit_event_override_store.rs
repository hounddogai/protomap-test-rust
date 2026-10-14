use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy audit event overrides by code.
pub trait LegacyAuditEventOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyAuditEventOverride>;
    fn save(&self, item: &LegacyAuditEventOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyAuditEventOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyAuditEventOverrideStore {
    pub items: RefCell<BTreeMap<String, LegacyAuditEventOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyAuditEventOverrideStore for MemoryLegacyAuditEventOverrideStore {
    fn find(&self, code: &str) -> Option<LegacyAuditEventOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyAuditEventOverride) {
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

    fn list(&self) -> Vec<LegacyAuditEventOverride> {
        let mut out: Vec<LegacyAuditEventOverride> = Vec::new();
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

pub fn new_memory_legacy_audit_event_override_store() -> Rc<dyn LegacyAuditEventOverrideStore> {
    Rc::new(MemoryLegacyAuditEventOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyAuditEventOverrideStore {
    pub inner: Rc<dyn LegacyAuditEventOverrideStore>,
    pub cache: RefCell<cache::TtlCache<LegacyAuditEventOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyAuditEventOverrideStore for CachedLegacyAuditEventOverrideStore {
    fn find(&self, code: &str) -> Option<LegacyAuditEventOverride> {
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

    fn save(&self, item: &LegacyAuditEventOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyAuditEventOverride> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_audit_event_override_store(
    inner: Rc<dyn LegacyAuditEventOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyAuditEventOverrideStore> {
    Rc::new(CachedLegacyAuditEventOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyAuditEventOverride>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy audit event overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventOverrideQuery {
    pub country: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyAuditEventOverrideQuery {
    pub fn matches(&self, item: &LegacyAuditEventOverride) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyAuditEventOverride]) -> Vec<LegacyAuditEventOverride> {
        let matched = collections::filter(items, |candidate: &LegacyAuditEventOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_audit_event_overrides(
    store: Rc<dyn LegacyAuditEventOverrideStore>,
    query: &LegacyAuditEventOverrideQuery,
) -> Vec<LegacyAuditEventOverride> {
    query.apply(&store.list())
}
