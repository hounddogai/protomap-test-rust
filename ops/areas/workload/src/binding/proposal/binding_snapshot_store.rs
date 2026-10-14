use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding snapshots by code.
pub trait BindingSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingSnapshot>;
    fn save(&self, item: &BindingSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingSnapshotStore {
    pub items: RefCell<BTreeMap<String, BindingSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingSnapshotStore for MemoryBindingSnapshotStore {
    fn find(&self, code: &str) -> Option<BindingSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingSnapshot) {
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

    fn list(&self) -> Vec<BindingSnapshot> {
        let mut out: Vec<BindingSnapshot> = Vec::new();
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

pub fn new_memory_binding_snapshot_store() -> Rc<dyn BindingSnapshotStore> {
    Rc::new(MemoryBindingSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingSnapshotStore {
    pub inner: Rc<dyn BindingSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<BindingSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingSnapshotStore for CachedBindingSnapshotStore {
    fn find(&self, code: &str) -> Option<BindingSnapshot> {
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

    fn save(&self, item: &BindingSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_binding_snapshot_store(
    inner: Rc<dyn BindingSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingSnapshotStore> {
    Rc::new(CachedBindingSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingSnapshot>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSnapshotQuery {
    pub label: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingSnapshotQuery {
    pub fn matches(&self, item: &BindingSnapshot) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingSnapshot]) -> Vec<BindingSnapshot> {
        let matched = collections::filter(items, |candidate: &BindingSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_snapshots(
    store: Rc<dyn BindingSnapshotStore>,
    query: &BindingSnapshotQuery,
) -> Vec<BindingSnapshot> {
    query.apply(&store.list())
}
