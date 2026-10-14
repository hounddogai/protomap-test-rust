use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction audits by code.
pub trait RedactionAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionAudit>;
    fn save(&self, item: &RedactionAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionAuditStore {
    pub items: RefCell<BTreeMap<String, RedactionAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionAuditStore for MemoryRedactionAuditStore {
    fn find(&self, code: &str) -> Option<RedactionAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionAudit) {
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

    fn list(&self) -> Vec<RedactionAudit> {
        let mut out: Vec<RedactionAudit> = Vec::new();
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

pub fn new_memory_redaction_audit_store() -> Rc<dyn RedactionAuditStore> {
    Rc::new(MemoryRedactionAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionAuditStore {
    pub inner: Rc<dyn RedactionAuditStore>,
    pub cache: RefCell<cache::TtlCache<RedactionAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionAuditStore for CachedRedactionAuditStore {
    fn find(&self, code: &str) -> Option<RedactionAudit> {
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

    fn save(&self, item: &RedactionAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionAudit> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_audit_store(
    inner: Rc<dyn RedactionAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionAuditStore> {
    Rc::new(CachedRedactionAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionAudit>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAuditQuery {
    pub title: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionAuditQuery {
    pub fn matches(&self, item: &RedactionAudit) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[RedactionAudit]) -> Vec<RedactionAudit> {
        let matched = collections::filter(items, |candidate: &RedactionAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_audits(store: Rc<dyn RedactionAuditStore>, query: &RedactionAuditQuery) -> Vec<RedactionAudit> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RedactionAuditStore> = new_memory_redaction_audit_store();
        let item = new_redaction_audit("code-956", "title-143");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
