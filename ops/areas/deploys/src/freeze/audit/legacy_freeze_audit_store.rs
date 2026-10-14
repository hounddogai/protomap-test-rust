use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy freeze audits by code.
pub trait LegacyFreezeAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyFreezeAudit>;
    fn save(&self, item: &LegacyFreezeAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyFreezeAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyFreezeAuditStore {
    pub items: RefCell<BTreeMap<String, LegacyFreezeAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyFreezeAuditStore for MemoryLegacyFreezeAuditStore {
    fn find(&self, code: &str) -> Option<LegacyFreezeAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyFreezeAudit) {
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

    fn list(&self) -> Vec<LegacyFreezeAudit> {
        let mut out: Vec<LegacyFreezeAudit> = Vec::new();
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

pub fn new_memory_legacy_freeze_audit_store() -> Rc<dyn LegacyFreezeAuditStore> {
    Rc::new(MemoryLegacyFreezeAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyFreezeAuditStore {
    pub inner: Rc<dyn LegacyFreezeAuditStore>,
    pub cache: RefCell<cache::TtlCache<LegacyFreezeAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyFreezeAuditStore for CachedLegacyFreezeAuditStore {
    fn find(&self, code: &str) -> Option<LegacyFreezeAudit> {
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

    fn save(&self, item: &LegacyFreezeAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyFreezeAudit> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_freeze_audit_store(
    inner: Rc<dyn LegacyFreezeAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyFreezeAuditStore> {
    Rc::new(CachedLegacyFreezeAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyFreezeAudit>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy freeze audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeAuditQuery {
    pub title: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyFreezeAuditQuery {
    pub fn matches(&self, item: &LegacyFreezeAudit) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyFreezeAudit]) -> Vec<LegacyFreezeAudit> {
        let matched = collections::filter(items, |candidate: &LegacyFreezeAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_freeze_audits(
    store: Rc<dyn LegacyFreezeAuditStore>,
    query: &LegacyFreezeAuditQuery,
) -> Vec<LegacyFreezeAudit> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyFreezeAuditStore> = new_memory_legacy_freeze_audit_store();
        let item = new_legacy_freeze_audit("code-876", "title-87");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
