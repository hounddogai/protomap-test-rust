use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps migration audits by code.
pub trait MigrationAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MigrationAudit>;
    fn save(&self, item: &MigrationAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MigrationAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMigrationAuditStore {
    pub items: RefCell<BTreeMap<String, MigrationAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl MigrationAuditStore for MemoryMigrationAuditStore {
    fn find(&self, code: &str) -> Option<MigrationAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MigrationAudit) {
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

    fn list(&self) -> Vec<MigrationAudit> {
        let mut out: Vec<MigrationAudit> = Vec::new();
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

pub fn new_memory_migration_audit_store() -> Rc<dyn MigrationAuditStore> {
    Rc::new(MemoryMigrationAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMigrationAuditStore {
    pub inner: Rc<dyn MigrationAuditStore>,
    pub cache: RefCell<cache::TtlCache<MigrationAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MigrationAuditStore for CachedMigrationAuditStore {
    fn find(&self, code: &str) -> Option<MigrationAudit> {
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

    fn save(&self, item: &MigrationAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MigrationAudit> {
        self.inner.list()
    }
}

pub fn new_cached_migration_audit_store(
    inner: Rc<dyn MigrationAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MigrationAuditStore> {
    Rc::new(CachedMigrationAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MigrationAudit>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects migration audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAuditQuery {
    pub label: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MigrationAuditQuery {
    pub fn matches(&self, item: &MigrationAudit) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MigrationAudit]) -> Vec<MigrationAudit> {
        let matched = collections::filter(items, |candidate: &MigrationAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_migration_audits(store: Rc<dyn MigrationAuditStore>, query: &MigrationAuditQuery) -> Vec<MigrationAudit> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MigrationAuditStore> = new_memory_migration_audit_store();
        let item = new_migration_audit("code-220", "label-157");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
