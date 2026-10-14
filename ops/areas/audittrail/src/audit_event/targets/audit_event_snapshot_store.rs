use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event snapshots by code.
pub trait AuditEventSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventSnapshot>;
    fn save(&self, item: &AuditEventSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventSnapshotStore {
    pub items: RefCell<BTreeMap<String, AuditEventSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventSnapshotStore for MemoryAuditEventSnapshotStore {
    fn find(&self, code: &str) -> Option<AuditEventSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventSnapshot) {
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

    fn list(&self) -> Vec<AuditEventSnapshot> {
        let mut out: Vec<AuditEventSnapshot> = Vec::new();
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

pub fn new_memory_audit_event_snapshot_store() -> Rc<dyn AuditEventSnapshotStore> {
    Rc::new(MemoryAuditEventSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventSnapshotStore {
    pub inner: Rc<dyn AuditEventSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventSnapshotStore for CachedAuditEventSnapshotStore {
    fn find(&self, code: &str) -> Option<AuditEventSnapshot> {
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

    fn save(&self, item: &AuditEventSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_snapshot_store(
    inner: Rc<dyn AuditEventSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventSnapshotStore> {
    Rc::new(CachedAuditEventSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventSnapshot>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSnapshotQuery {
    pub label: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventSnapshotQuery {
    pub fn matches(&self, item: &AuditEventSnapshot) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AuditEventSnapshot]) -> Vec<AuditEventSnapshot> {
        let matched = collections::filter(items, |candidate: &AuditEventSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_snapshots(
    store: Rc<dyn AuditEventSnapshotStore>,
    query: &AuditEventSnapshotQuery,
) -> Vec<AuditEventSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AuditEventSnapshotStore> = new_memory_audit_event_snapshot_store();
        let item = new_audit_event_snapshot("code-420", "label-433");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
