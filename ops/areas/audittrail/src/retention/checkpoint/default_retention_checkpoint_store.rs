use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default retention checkpoints by code.
pub trait DefaultRetentionCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultRetentionCheckpoint>;
    fn save(&self, item: &DefaultRetentionCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultRetentionCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultRetentionCheckpointStore {
    pub items: RefCell<BTreeMap<String, DefaultRetentionCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultRetentionCheckpointStore for MemoryDefaultRetentionCheckpointStore {
    fn find(&self, code: &str) -> Option<DefaultRetentionCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultRetentionCheckpoint) {
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

    fn list(&self) -> Vec<DefaultRetentionCheckpoint> {
        let mut out: Vec<DefaultRetentionCheckpoint> = Vec::new();
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

pub fn new_memory_default_retention_checkpoint_store() -> Rc<dyn DefaultRetentionCheckpointStore> {
    Rc::new(MemoryDefaultRetentionCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultRetentionCheckpointStore {
    pub inner: Rc<dyn DefaultRetentionCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<DefaultRetentionCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultRetentionCheckpointStore for CachedDefaultRetentionCheckpointStore {
    fn find(&self, code: &str) -> Option<DefaultRetentionCheckpoint> {
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

    fn save(&self, item: &DefaultRetentionCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultRetentionCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_default_retention_checkpoint_store(
    inner: Rc<dyn DefaultRetentionCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultRetentionCheckpointStore> {
    Rc::new(CachedDefaultRetentionCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultRetentionCheckpoint>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default retention checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionCheckpointQuery {
    pub category: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultRetentionCheckpointQuery {
    pub fn matches(&self, item: &DefaultRetentionCheckpoint) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[DefaultRetentionCheckpoint]) -> Vec<DefaultRetentionCheckpoint> {
        let matched = collections::filter(items, |candidate: &DefaultRetentionCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_retention_checkpoints(
    store: Rc<dyn DefaultRetentionCheckpointStore>,
    query: &DefaultRetentionCheckpointQuery,
) -> Vec<DefaultRetentionCheckpoint> {
    query.apply(&store.list())
}
