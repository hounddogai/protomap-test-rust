use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event targets by code.
pub trait AuditEventTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventTarget>;
    fn save(&self, item: &AuditEventTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventTargetStore {
    pub items: RefCell<BTreeMap<String, AuditEventTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventTargetStore for MemoryAuditEventTargetStore {
    fn find(&self, code: &str) -> Option<AuditEventTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventTarget) {
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

    fn list(&self) -> Vec<AuditEventTarget> {
        let mut out: Vec<AuditEventTarget> = Vec::new();
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

pub fn new_memory_audit_event_target_store() -> Rc<dyn AuditEventTargetStore> {
    Rc::new(MemoryAuditEventTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventTargetStore {
    pub inner: Rc<dyn AuditEventTargetStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventTargetStore for CachedAuditEventTargetStore {
    fn find(&self, code: &str) -> Option<AuditEventTarget> {
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

    fn save(&self, item: &AuditEventTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventTarget> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_target_store(
    inner: Rc<dyn AuditEventTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventTargetStore> {
    Rc::new(CachedAuditEventTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventTarget>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTargetQuery {
    pub cluster: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventTargetQuery {
    pub fn matches(&self, item: &AuditEventTarget) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[AuditEventTarget]) -> Vec<AuditEventTarget> {
        let matched = collections::filter(items, |candidate: &AuditEventTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_targets(
    store: Rc<dyn AuditEventTargetStore>,
    query: &AuditEventTargetQuery,
) -> Vec<AuditEventTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AuditEventTargetStore> = new_memory_audit_event_target_store();
        let item = new_audit_event_target("code-163", "label-357");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
