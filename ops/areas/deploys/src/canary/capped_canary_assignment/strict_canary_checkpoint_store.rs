use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict canary checkpoints by code.
pub trait StrictCanaryCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictCanaryCheckpoint>;
    fn save(&self, item: &StrictCanaryCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictCanaryCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictCanaryCheckpointStore {
    pub items: RefCell<BTreeMap<String, StrictCanaryCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictCanaryCheckpointStore for MemoryStrictCanaryCheckpointStore {
    fn find(&self, code: &str) -> Option<StrictCanaryCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictCanaryCheckpoint) {
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

    fn list(&self) -> Vec<StrictCanaryCheckpoint> {
        let mut out: Vec<StrictCanaryCheckpoint> = Vec::new();
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

pub fn new_memory_strict_canary_checkpoint_store() -> Rc<dyn StrictCanaryCheckpointStore> {
    Rc::new(MemoryStrictCanaryCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictCanaryCheckpointStore {
    pub inner: Rc<dyn StrictCanaryCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<StrictCanaryCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictCanaryCheckpointStore for CachedStrictCanaryCheckpointStore {
    fn find(&self, code: &str) -> Option<StrictCanaryCheckpoint> {
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

    fn save(&self, item: &StrictCanaryCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictCanaryCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_strict_canary_checkpoint_store(
    inner: Rc<dyn StrictCanaryCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictCanaryCheckpointStore> {
    Rc::new(CachedStrictCanaryCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictCanaryCheckpoint>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict canary checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryCheckpointQuery {
    pub name: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictCanaryCheckpointQuery {
    pub fn matches(&self, item: &StrictCanaryCheckpoint) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictCanaryCheckpoint]) -> Vec<StrictCanaryCheckpoint> {
        let matched = collections::filter(items, |candidate: &StrictCanaryCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_canary_checkpoints(
    store: Rc<dyn StrictCanaryCheckpointStore>,
    query: &StrictCanaryCheckpointQuery,
) -> Vec<StrictCanaryCheckpoint> {
    query.apply(&store.list())
}
