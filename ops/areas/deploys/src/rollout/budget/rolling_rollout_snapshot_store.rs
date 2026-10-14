use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling rollout snapshots by code.
pub trait RollingRolloutSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingRolloutSnapshot>;
    fn save(&self, item: &RollingRolloutSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingRolloutSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingRolloutSnapshotStore {
    pub items: RefCell<BTreeMap<String, RollingRolloutSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingRolloutSnapshotStore for MemoryRollingRolloutSnapshotStore {
    fn find(&self, code: &str) -> Option<RollingRolloutSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingRolloutSnapshot) {
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

    fn list(&self) -> Vec<RollingRolloutSnapshot> {
        let mut out: Vec<RollingRolloutSnapshot> = Vec::new();
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

pub fn new_memory_rolling_rollout_snapshot_store() -> Rc<dyn RollingRolloutSnapshotStore> {
    Rc::new(MemoryRollingRolloutSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingRolloutSnapshotStore {
    pub inner: Rc<dyn RollingRolloutSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<RollingRolloutSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingRolloutSnapshotStore for CachedRollingRolloutSnapshotStore {
    fn find(&self, code: &str) -> Option<RollingRolloutSnapshot> {
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

    fn save(&self, item: &RollingRolloutSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingRolloutSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_rollout_snapshot_store(
    inner: Rc<dyn RollingRolloutSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingRolloutSnapshotStore> {
    Rc::new(CachedRollingRolloutSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingRolloutSnapshot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling rollout snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutSnapshotQuery {
    pub channel: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingRolloutSnapshotQuery {
    pub fn matches(&self, item: &RollingRolloutSnapshot) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingRolloutSnapshot]) -> Vec<RollingRolloutSnapshot> {
        let matched = collections::filter(items, |candidate: &RollingRolloutSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_rollout_snapshots(
    store: Rc<dyn RollingRolloutSnapshotStore>,
    query: &RollingRolloutSnapshotQuery,
) -> Vec<RollingRolloutSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingRolloutSnapshotStore> = new_memory_rolling_rollout_snapshot_store();
        let item = new_rolling_rollout_snapshot("code-610", "label-309");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
