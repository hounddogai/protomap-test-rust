use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential snapshots by code.
pub trait CredentialSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialSnapshot>;
    fn save(&self, item: &CredentialSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialSnapshotStore {
    pub items: RefCell<BTreeMap<String, CredentialSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialSnapshotStore for MemoryCredentialSnapshotStore {
    fn find(&self, code: &str) -> Option<CredentialSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialSnapshot) {
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

    fn list(&self) -> Vec<CredentialSnapshot> {
        let mut out: Vec<CredentialSnapshot> = Vec::new();
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

pub fn new_memory_credential_snapshot_store() -> Rc<dyn CredentialSnapshotStore> {
    Rc::new(MemoryCredentialSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialSnapshotStore {
    pub inner: Rc<dyn CredentialSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<CredentialSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialSnapshotStore for CachedCredentialSnapshotStore {
    fn find(&self, code: &str) -> Option<CredentialSnapshot> {
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

    fn save(&self, item: &CredentialSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_credential_snapshot_store(
    inner: Rc<dyn CredentialSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialSnapshotStore> {
    Rc::new(CachedCredentialSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialSnapshot>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshotQuery {
    pub namespace: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialSnapshotQuery {
    pub fn matches(&self, item: &CredentialSnapshot) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[CredentialSnapshot]) -> Vec<CredentialSnapshot> {
        let matched = collections::filter(items, |candidate: &CredentialSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_snapshots(
    store: Rc<dyn CredentialSnapshotStore>,
    query: &CredentialSnapshotQuery,
) -> Vec<CredentialSnapshot> {
    query.apply(&store.list())
}
