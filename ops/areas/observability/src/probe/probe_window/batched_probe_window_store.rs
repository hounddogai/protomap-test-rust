use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched probe windows by code.
pub trait BatchedProbeWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedProbeWindow>;
    fn save(&self, item: &BatchedProbeWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedProbeWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedProbeWindowStore {
    pub items: RefCell<BTreeMap<String, BatchedProbeWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedProbeWindowStore for MemoryBatchedProbeWindowStore {
    fn find(&self, code: &str) -> Option<BatchedProbeWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedProbeWindow) {
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

    fn list(&self) -> Vec<BatchedProbeWindow> {
        let mut out: Vec<BatchedProbeWindow> = Vec::new();
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

pub fn new_memory_batched_probe_window_store() -> Rc<dyn BatchedProbeWindowStore> {
    Rc::new(MemoryBatchedProbeWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedProbeWindowStore {
    pub inner: Rc<dyn BatchedProbeWindowStore>,
    pub cache: RefCell<cache::TtlCache<BatchedProbeWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedProbeWindowStore for CachedBatchedProbeWindowStore {
    fn find(&self, code: &str) -> Option<BatchedProbeWindow> {
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

    fn save(&self, item: &BatchedProbeWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedProbeWindow> {
        self.inner.list()
    }
}

pub fn new_cached_batched_probe_window_store(
    inner: Rc<dyn BatchedProbeWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedProbeWindowStore> {
    Rc::new(CachedBatchedProbeWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedProbeWindow>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched probe windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeWindowQuery {
    pub name: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedProbeWindowQuery {
    pub fn matches(&self, item: &BatchedProbeWindow) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedProbeWindow]) -> Vec<BatchedProbeWindow> {
        let matched = collections::filter(items, |candidate: &BatchedProbeWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_probe_windows(
    store: Rc<dyn BatchedProbeWindowStore>,
    query: &BatchedProbeWindowQuery,
) -> Vec<BatchedProbeWindow> {
    query.apply(&store.list())
}
