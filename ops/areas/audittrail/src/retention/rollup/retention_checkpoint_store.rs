use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention checkpoints by code.
pub trait RetentionCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionCheckpoint>;
    fn save(&self, item: &RetentionCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionCheckpointStore {
    pub items: RefCell<BTreeMap<String, RetentionCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionCheckpointStore for MemoryRetentionCheckpointStore {
    fn find(&self, code: &str) -> Option<RetentionCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionCheckpoint) {
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

    fn list(&self) -> Vec<RetentionCheckpoint> {
        let mut out: Vec<RetentionCheckpoint> = Vec::new();
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

pub fn new_memory_retention_checkpoint_store() -> Rc<dyn RetentionCheckpointStore> {
    Rc::new(MemoryRetentionCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionCheckpointStore {
    pub inner: Rc<dyn RetentionCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<RetentionCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionCheckpointStore for CachedRetentionCheckpointStore {
    fn find(&self, code: &str) -> Option<RetentionCheckpoint> {
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

    fn save(&self, item: &RetentionCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_retention_checkpoint_store(
    inner: Rc<dyn RetentionCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionCheckpointStore> {
    Rc::new(CachedRetentionCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionCheckpoint>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCheckpointQuery {
    pub source: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionCheckpointQuery {
    pub fn matches(&self, item: &RetentionCheckpoint) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RetentionCheckpoint]) -> Vec<RetentionCheckpoint> {
        let matched = collections::filter(items, |candidate: &RetentionCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_checkpoints(
    store: Rc<dyn RetentionCheckpointStore>,
    query: &RetentionCheckpointQuery,
) -> Vec<RetentionCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RetentionCheckpointStore> = new_memory_retention_checkpoint_store();
        let item = new_retention_checkpoint("code-811", "title-664");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
