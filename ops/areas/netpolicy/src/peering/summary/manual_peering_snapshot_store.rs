use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual peering snapshots by code.
pub trait ManualPeeringSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualPeeringSnapshot>;
    fn save(&self, item: &ManualPeeringSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualPeeringSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualPeeringSnapshotStore {
    pub items: RefCell<BTreeMap<String, ManualPeeringSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualPeeringSnapshotStore for MemoryManualPeeringSnapshotStore {
    fn find(&self, code: &str) -> Option<ManualPeeringSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualPeeringSnapshot) {
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

    fn list(&self) -> Vec<ManualPeeringSnapshot> {
        let mut out: Vec<ManualPeeringSnapshot> = Vec::new();
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

pub fn new_memory_manual_peering_snapshot_store() -> Rc<dyn ManualPeeringSnapshotStore> {
    Rc::new(MemoryManualPeeringSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualPeeringSnapshotStore {
    pub inner: Rc<dyn ManualPeeringSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<ManualPeeringSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualPeeringSnapshotStore for CachedManualPeeringSnapshotStore {
    fn find(&self, code: &str) -> Option<ManualPeeringSnapshot> {
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

    fn save(&self, item: &ManualPeeringSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualPeeringSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_manual_peering_snapshot_store(
    inner: Rc<dyn ManualPeeringSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualPeeringSnapshotStore> {
    Rc::new(CachedManualPeeringSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualPeeringSnapshot>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual peering snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPeeringSnapshotQuery {
    pub channel: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualPeeringSnapshotQuery {
    pub fn matches(&self, item: &ManualPeeringSnapshot) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualPeeringSnapshot]) -> Vec<ManualPeeringSnapshot> {
        let matched = collections::filter(items, |candidate: &ManualPeeringSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_peering_snapshots(
    store: Rc<dyn ManualPeeringSnapshotStore>,
    query: &ManualPeeringSnapshotQuery,
) -> Vec<ManualPeeringSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualPeeringSnapshotStore> = new_memory_manual_peering_snapshot_store();
        let item = new_manual_peering_snapshot("code-153", "label-914");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
