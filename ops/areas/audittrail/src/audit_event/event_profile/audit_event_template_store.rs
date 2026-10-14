use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event templates by code.
pub trait AuditEventTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventTemplate>;
    fn save(&self, item: &AuditEventTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventTemplateStore {
    pub items: RefCell<BTreeMap<String, AuditEventTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventTemplateStore for MemoryAuditEventTemplateStore {
    fn find(&self, code: &str) -> Option<AuditEventTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventTemplate) {
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

    fn list(&self) -> Vec<AuditEventTemplate> {
        let mut out: Vec<AuditEventTemplate> = Vec::new();
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

pub fn new_memory_audit_event_template_store() -> Rc<dyn AuditEventTemplateStore> {
    Rc::new(MemoryAuditEventTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventTemplateStore {
    pub inner: Rc<dyn AuditEventTemplateStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventTemplateStore for CachedAuditEventTemplateStore {
    fn find(&self, code: &str) -> Option<AuditEventTemplate> {
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

    fn save(&self, item: &AuditEventTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_template_store(
    inner: Rc<dyn AuditEventTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventTemplateStore> {
    Rc::new(CachedAuditEventTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventTemplate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTemplateQuery {
    pub name: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventTemplateQuery {
    pub fn matches(&self, item: &AuditEventTemplate) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AuditEventTemplate]) -> Vec<AuditEventTemplate> {
        let matched = collections::filter(items, |candidate: &AuditEventTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_templates(
    store: Rc<dyn AuditEventTemplateStore>,
    query: &AuditEventTemplateQuery,
) -> Vec<AuditEventTemplate> {
    query.apply(&store.list())
}
