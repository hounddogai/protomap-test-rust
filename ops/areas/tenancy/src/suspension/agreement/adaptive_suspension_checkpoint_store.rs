use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive suspension checkpoints by code.
pub trait AdaptiveSuspensionCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveSuspensionCheckpoint>;
    fn save(&self, item: &AdaptiveSuspensionCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveSuspensionCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveSuspensionCheckpointStore {
    pub items: RefCell<BTreeMap<String, AdaptiveSuspensionCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveSuspensionCheckpointStore for MemoryAdaptiveSuspensionCheckpointStore {
    fn find(&self, code: &str) -> Option<AdaptiveSuspensionCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveSuspensionCheckpoint) {
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

    fn list(&self) -> Vec<AdaptiveSuspensionCheckpoint> {
        let mut out: Vec<AdaptiveSuspensionCheckpoint> = Vec::new();
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

pub fn new_memory_adaptive_suspension_checkpoint_store() -> Rc<dyn AdaptiveSuspensionCheckpointStore> {
    Rc::new(MemoryAdaptiveSuspensionCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveSuspensionCheckpointStore {
    pub inner: Rc<dyn AdaptiveSuspensionCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveSuspensionCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveSuspensionCheckpointStore for CachedAdaptiveSuspensionCheckpointStore {
    fn find(&self, code: &str) -> Option<AdaptiveSuspensionCheckpoint> {
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

    fn save(&self, item: &AdaptiveSuspensionCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveSuspensionCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_suspension_checkpoint_store(
    inner: Rc<dyn AdaptiveSuspensionCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveSuspensionCheckpointStore> {
    Rc::new(CachedAdaptiveSuspensionCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveSuspensionCheckpoint>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive suspension checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionCheckpointQuery {
    pub name: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveSuspensionCheckpointQuery {
    pub fn matches(&self, item: &AdaptiveSuspensionCheckpoint) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveSuspensionCheckpoint]) -> Vec<AdaptiveSuspensionCheckpoint> {
        let matched = collections::filter(items, |candidate: &AdaptiveSuspensionCheckpoint| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_suspension_checkpoints(
    store: Rc<dyn AdaptiveSuspensionCheckpointStore>,
    query: &AdaptiveSuspensionCheckpointQuery,
) -> Vec<AdaptiveSuspensionCheckpoint> {
    query.apply(&store.list())
}
