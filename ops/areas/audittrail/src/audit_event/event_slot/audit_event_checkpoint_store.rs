use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event checkpoints by code.
pub trait AuditEventCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventCheckpoint>;
    fn save(&self, item: &AuditEventCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventCheckpointStore {
    pub items: RefCell<BTreeMap<String, AuditEventCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventCheckpointStore for MemoryAuditEventCheckpointStore {
    fn find(&self, code: &str) -> Option<AuditEventCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventCheckpoint) {
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

    fn list(&self) -> Vec<AuditEventCheckpoint> {
        let mut out: Vec<AuditEventCheckpoint> = Vec::new();
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

pub fn new_memory_audit_event_checkpoint_store() -> Rc<dyn AuditEventCheckpointStore> {
    Rc::new(MemoryAuditEventCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventCheckpointStore {
    pub inner: Rc<dyn AuditEventCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventCheckpointStore for CachedAuditEventCheckpointStore {
    fn find(&self, code: &str) -> Option<AuditEventCheckpoint> {
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

    fn save(&self, item: &AuditEventCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_checkpoint_store(
    inner: Rc<dyn AuditEventCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventCheckpointStore> {
    Rc::new(CachedAuditEventCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventCheckpoint>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCheckpointQuery {
    pub unit: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventCheckpointQuery {
    pub fn matches(&self, item: &AuditEventCheckpoint) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AuditEventCheckpoint]) -> Vec<AuditEventCheckpoint> {
        let matched = collections::filter(items, |candidate: &AuditEventCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_checkpoints(
    store: Rc<dyn AuditEventCheckpointStore>,
    query: &AuditEventCheckpointQuery,
) -> Vec<AuditEventCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AuditEventCheckpointStore> = new_memory_audit_event_checkpoint_store();
        let item = new_audit_event_checkpoint("code-974", "title-652");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
