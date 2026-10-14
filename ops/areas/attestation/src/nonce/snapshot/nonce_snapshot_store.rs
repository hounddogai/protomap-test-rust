use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps nonce snapshots by code.
pub trait NonceSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<NonceSnapshot>;
    fn save(&self, item: &NonceSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<NonceSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNonceSnapshotStore {
    pub items: RefCell<BTreeMap<String, NonceSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl NonceSnapshotStore for MemoryNonceSnapshotStore {
    fn find(&self, code: &str) -> Option<NonceSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &NonceSnapshot) {
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

    fn list(&self) -> Vec<NonceSnapshot> {
        let mut out: Vec<NonceSnapshot> = Vec::new();
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

pub fn new_memory_nonce_snapshot_store() -> Rc<dyn NonceSnapshotStore> {
    Rc::new(MemoryNonceSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedNonceSnapshotStore {
    pub inner: Rc<dyn NonceSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<NonceSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl NonceSnapshotStore for CachedNonceSnapshotStore {
    fn find(&self, code: &str) -> Option<NonceSnapshot> {
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

    fn save(&self, item: &NonceSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<NonceSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_nonce_snapshot_store(
    inner: Rc<dyn NonceSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn NonceSnapshotStore> {
    Rc::new(CachedNonceSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<NonceSnapshot>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects nonce snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSnapshotQuery {
    pub unit: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl NonceSnapshotQuery {
    pub fn matches(&self, item: &NonceSnapshot) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[NonceSnapshot]) -> Vec<NonceSnapshot> {
        let matched = collections::filter(items, |candidate: &NonceSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_nonce_snapshots(store: Rc<dyn NonceSnapshotStore>, query: &NonceSnapshotQuery) -> Vec<NonceSnapshot> {
    query.apply(&store.list())
}
