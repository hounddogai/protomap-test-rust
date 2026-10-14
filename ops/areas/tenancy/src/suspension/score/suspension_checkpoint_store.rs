use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension checkpoints by code.
pub trait SuspensionCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionCheckpoint>;
    fn save(&self, item: &SuspensionCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionCheckpointStore {
    pub items: RefCell<BTreeMap<String, SuspensionCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionCheckpointStore for MemorySuspensionCheckpointStore {
    fn find(&self, code: &str) -> Option<SuspensionCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionCheckpoint) {
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

    fn list(&self) -> Vec<SuspensionCheckpoint> {
        let mut out: Vec<SuspensionCheckpoint> = Vec::new();
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

pub fn new_memory_suspension_checkpoint_store() -> Rc<dyn SuspensionCheckpointStore> {
    Rc::new(MemorySuspensionCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionCheckpointStore {
    pub inner: Rc<dyn SuspensionCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionCheckpointStore for CachedSuspensionCheckpointStore {
    fn find(&self, code: &str) -> Option<SuspensionCheckpoint> {
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

    fn save(&self, item: &SuspensionCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_checkpoint_store(
    inner: Rc<dyn SuspensionCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionCheckpointStore> {
    Rc::new(CachedSuspensionCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionCheckpoint>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionCheckpointQuery {
    pub name: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionCheckpointQuery {
    pub fn matches(&self, item: &SuspensionCheckpoint) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SuspensionCheckpoint]) -> Vec<SuspensionCheckpoint> {
        let matched = collections::filter(items, |candidate: &SuspensionCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_checkpoints(
    store: Rc<dyn SuspensionCheckpointStore>,
    query: &SuspensionCheckpointQuery,
) -> Vec<SuspensionCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SuspensionCheckpointStore> = new_memory_suspension_checkpoint_store();
        let item = new_suspension_checkpoint("code-456", "name-283");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
