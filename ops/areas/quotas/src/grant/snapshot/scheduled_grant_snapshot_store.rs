use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled grant snapshots by code.
pub trait ScheduledGrantSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledGrantSnapshot>;
    fn save(&self, item: &ScheduledGrantSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledGrantSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledGrantSnapshotStore {
    pub items: RefCell<BTreeMap<String, ScheduledGrantSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledGrantSnapshotStore for MemoryScheduledGrantSnapshotStore {
    fn find(&self, code: &str) -> Option<ScheduledGrantSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledGrantSnapshot) {
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

    fn list(&self) -> Vec<ScheduledGrantSnapshot> {
        let mut out: Vec<ScheduledGrantSnapshot> = Vec::new();
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

pub fn new_memory_scheduled_grant_snapshot_store() -> Rc<dyn ScheduledGrantSnapshotStore> {
    Rc::new(MemoryScheduledGrantSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledGrantSnapshotStore {
    pub inner: Rc<dyn ScheduledGrantSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledGrantSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledGrantSnapshotStore for CachedScheduledGrantSnapshotStore {
    fn find(&self, code: &str) -> Option<ScheduledGrantSnapshot> {
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

    fn save(&self, item: &ScheduledGrantSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledGrantSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_grant_snapshot_store(
    inner: Rc<dyn ScheduledGrantSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledGrantSnapshotStore> {
    Rc::new(CachedScheduledGrantSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledGrantSnapshot>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled grant snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantSnapshotQuery {
    pub locale: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledGrantSnapshotQuery {
    pub fn matches(&self, item: &ScheduledGrantSnapshot) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledGrantSnapshot]) -> Vec<ScheduledGrantSnapshot> {
        let matched = collections::filter(items, |candidate: &ScheduledGrantSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_grant_snapshots(
    store: Rc<dyn ScheduledGrantSnapshotStore>,
    query: &ScheduledGrantSnapshotQuery,
) -> Vec<ScheduledGrantSnapshot> {
    query.apply(&store.list())
}
