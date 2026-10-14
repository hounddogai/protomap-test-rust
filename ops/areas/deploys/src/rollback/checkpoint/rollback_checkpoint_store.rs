use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback checkpoints by code.
pub trait RollbackCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackCheckpoint>;
    fn save(&self, item: &RollbackCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackCheckpointStore {
    pub items: RefCell<BTreeMap<String, RollbackCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackCheckpointStore for MemoryRollbackCheckpointStore {
    fn find(&self, code: &str) -> Option<RollbackCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackCheckpoint) {
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

    fn list(&self) -> Vec<RollbackCheckpoint> {
        let mut out: Vec<RollbackCheckpoint> = Vec::new();
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

pub fn new_memory_rollback_checkpoint_store() -> Rc<dyn RollbackCheckpointStore> {
    Rc::new(MemoryRollbackCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackCheckpointStore {
    pub inner: Rc<dyn RollbackCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<RollbackCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackCheckpointStore for CachedRollbackCheckpointStore {
    fn find(&self, code: &str) -> Option<RollbackCheckpoint> {
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

    fn save(&self, item: &RollbackCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_checkpoint_store(
    inner: Rc<dyn RollbackCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackCheckpointStore> {
    Rc::new(CachedRollbackCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackCheckpoint>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCheckpointQuery {
    pub name: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackCheckpointQuery {
    pub fn matches(&self, item: &RollbackCheckpoint) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollbackCheckpoint]) -> Vec<RollbackCheckpoint> {
        let matched = collections::filter(items, |candidate: &RollbackCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_checkpoints(
    store: Rc<dyn RollbackCheckpointStore>,
    query: &RollbackCheckpointQuery,
) -> Vec<RollbackCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollbackCheckpointStore> = new_memory_rollback_checkpoint_store();
        let item = new_rollback_checkpoint("code-796", "name-528");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
