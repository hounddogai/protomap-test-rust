use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollout snapshots by code.
pub trait RolloutSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RolloutSnapshot>;
    fn save(&self, item: &RolloutSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RolloutSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRolloutSnapshotStore {
    pub items: RefCell<BTreeMap<String, RolloutSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl RolloutSnapshotStore for MemoryRolloutSnapshotStore {
    fn find(&self, code: &str) -> Option<RolloutSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RolloutSnapshot) {
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

    fn list(&self) -> Vec<RolloutSnapshot> {
        let mut out: Vec<RolloutSnapshot> = Vec::new();
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

pub fn new_memory_rollout_snapshot_store() -> Rc<dyn RolloutSnapshotStore> {
    Rc::new(MemoryRolloutSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRolloutSnapshotStore {
    pub inner: Rc<dyn RolloutSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<RolloutSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RolloutSnapshotStore for CachedRolloutSnapshotStore {
    fn find(&self, code: &str) -> Option<RolloutSnapshot> {
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

    fn save(&self, item: &RolloutSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RolloutSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_rollout_snapshot_store(
    inner: Rc<dyn RolloutSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RolloutSnapshotStore> {
    Rc::new(CachedRolloutSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RolloutSnapshot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollout snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSnapshotQuery {
    pub name: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RolloutSnapshotQuery {
    pub fn matches(&self, item: &RolloutSnapshot) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RolloutSnapshot]) -> Vec<RolloutSnapshot> {
        let matched = collections::filter(items, |candidate: &RolloutSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollout_snapshots(
    store: Rc<dyn RolloutSnapshotStore>,
    query: &RolloutSnapshotQuery,
) -> Vec<RolloutSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RolloutSnapshotStore> = new_memory_rollout_snapshot_store();
        let item = new_rollout_snapshot("code-15", "name-486");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
