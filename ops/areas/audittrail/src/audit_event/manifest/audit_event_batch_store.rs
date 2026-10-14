use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event batches by code.
pub trait AuditEventBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventBatch>;
    fn save(&self, item: &AuditEventBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventBatchStore {
    pub items: RefCell<BTreeMap<String, AuditEventBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventBatchStore for MemoryAuditEventBatchStore {
    fn find(&self, code: &str) -> Option<AuditEventBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventBatch) {
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

    fn list(&self) -> Vec<AuditEventBatch> {
        let mut out: Vec<AuditEventBatch> = Vec::new();
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

pub fn new_memory_audit_event_batch_store() -> Rc<dyn AuditEventBatchStore> {
    Rc::new(MemoryAuditEventBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventBatchStore {
    pub inner: Rc<dyn AuditEventBatchStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventBatchStore for CachedAuditEventBatchStore {
    fn find(&self, code: &str) -> Option<AuditEventBatch> {
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

    fn save(&self, item: &AuditEventBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventBatch> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_batch_store(
    inner: Rc<dyn AuditEventBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventBatchStore> {
    Rc::new(CachedAuditEventBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventBatch>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBatchQuery {
    pub title: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventBatchQuery {
    pub fn matches(&self, item: &AuditEventBatch) -> bool {
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

    pub fn apply(&self, items: &[AuditEventBatch]) -> Vec<AuditEventBatch> {
        let matched = collections::filter(items, |candidate: &AuditEventBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_batches(
    store: Rc<dyn AuditEventBatchStore>,
    query: &AuditEventBatchQuery,
) -> Vec<AuditEventBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AuditEventBatchStore> = new_memory_audit_event_batch_store();
        let item = new_audit_event_batch("code-272", "title-959");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
