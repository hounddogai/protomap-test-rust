use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default rollback snapshots by code.
pub trait DefaultRollbackSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultRollbackSnapshot>;
    fn save(&self, item: &DefaultRollbackSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultRollbackSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultRollbackSnapshotStore {
    pub items: RefCell<BTreeMap<String, DefaultRollbackSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultRollbackSnapshotStore for MemoryDefaultRollbackSnapshotStore {
    fn find(&self, code: &str) -> Option<DefaultRollbackSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultRollbackSnapshot) {
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

    fn list(&self) -> Vec<DefaultRollbackSnapshot> {
        let mut out: Vec<DefaultRollbackSnapshot> = Vec::new();
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

pub fn new_memory_default_rollback_snapshot_store() -> Rc<dyn DefaultRollbackSnapshotStore> {
    Rc::new(MemoryDefaultRollbackSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultRollbackSnapshotStore {
    pub inner: Rc<dyn DefaultRollbackSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<DefaultRollbackSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultRollbackSnapshotStore for CachedDefaultRollbackSnapshotStore {
    fn find(&self, code: &str) -> Option<DefaultRollbackSnapshot> {
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

    fn save(&self, item: &DefaultRollbackSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultRollbackSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_default_rollback_snapshot_store(
    inner: Rc<dyn DefaultRollbackSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultRollbackSnapshotStore> {
    Rc::new(CachedDefaultRollbackSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultRollbackSnapshot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default rollback snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackSnapshotQuery {
    pub region: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultRollbackSnapshotQuery {
    pub fn matches(&self, item: &DefaultRollbackSnapshot) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultRollbackSnapshot]) -> Vec<DefaultRollbackSnapshot> {
        let matched = collections::filter(items, |candidate: &DefaultRollbackSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_rollback_snapshots(
    store: Rc<dyn DefaultRollbackSnapshotStore>,
    query: &DefaultRollbackSnapshotQuery,
) -> Vec<DefaultRollbackSnapshot> {
    query.apply(&store.list())
}
