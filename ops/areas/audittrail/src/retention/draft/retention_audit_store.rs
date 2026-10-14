use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention audits by code.
pub trait RetentionAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionAudit>;
    fn save(&self, item: &RetentionAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionAuditStore {
    pub items: RefCell<BTreeMap<String, RetentionAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionAuditStore for MemoryRetentionAuditStore {
    fn find(&self, code: &str) -> Option<RetentionAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionAudit) {
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

    fn list(&self) -> Vec<RetentionAudit> {
        let mut out: Vec<RetentionAudit> = Vec::new();
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

pub fn new_memory_retention_audit_store() -> Rc<dyn RetentionAuditStore> {
    Rc::new(MemoryRetentionAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionAuditStore {
    pub inner: Rc<dyn RetentionAuditStore>,
    pub cache: RefCell<cache::TtlCache<RetentionAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionAuditStore for CachedRetentionAuditStore {
    fn find(&self, code: &str) -> Option<RetentionAudit> {
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

    fn save(&self, item: &RetentionAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionAudit> {
        self.inner.list()
    }
}

pub fn new_cached_retention_audit_store(
    inner: Rc<dyn RetentionAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionAuditStore> {
    Rc::new(CachedRetentionAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionAudit>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAuditQuery {
    pub label: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionAuditQuery {
    pub fn matches(&self, item: &RetentionAudit) -> bool {
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

    pub fn apply(&self, items: &[RetentionAudit]) -> Vec<RetentionAudit> {
        let matched = collections::filter(items, |candidate: &RetentionAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_audits(store: Rc<dyn RetentionAuditStore>, query: &RetentionAuditQuery) -> Vec<RetentionAudit> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RetentionAuditStore> = new_memory_retention_audit_store();
        let item = new_retention_audit("code-784", "label-591");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
