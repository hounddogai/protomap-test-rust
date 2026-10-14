use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched redaction windows by code.
pub trait BatchedRedactionWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedRedactionWindow>;
    fn save(&self, item: &BatchedRedactionWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedRedactionWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedRedactionWindowStore {
    pub items: RefCell<BTreeMap<String, BatchedRedactionWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedRedactionWindowStore for MemoryBatchedRedactionWindowStore {
    fn find(&self, code: &str) -> Option<BatchedRedactionWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedRedactionWindow) {
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

    fn list(&self) -> Vec<BatchedRedactionWindow> {
        let mut out: Vec<BatchedRedactionWindow> = Vec::new();
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

pub fn new_memory_batched_redaction_window_store() -> Rc<dyn BatchedRedactionWindowStore> {
    Rc::new(MemoryBatchedRedactionWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedRedactionWindowStore {
    pub inner: Rc<dyn BatchedRedactionWindowStore>,
    pub cache: RefCell<cache::TtlCache<BatchedRedactionWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedRedactionWindowStore for CachedBatchedRedactionWindowStore {
    fn find(&self, code: &str) -> Option<BatchedRedactionWindow> {
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

    fn save(&self, item: &BatchedRedactionWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedRedactionWindow> {
        self.inner.list()
    }
}

pub fn new_cached_batched_redaction_window_store(
    inner: Rc<dyn BatchedRedactionWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedRedactionWindowStore> {
    Rc::new(CachedBatchedRedactionWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedRedactionWindow>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched redaction windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionWindowQuery {
    pub cluster: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedRedactionWindowQuery {
    pub fn matches(&self, item: &BatchedRedactionWindow) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedRedactionWindow]) -> Vec<BatchedRedactionWindow> {
        let matched = collections::filter(items, |candidate: &BatchedRedactionWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_redaction_windows(
    store: Rc<dyn BatchedRedactionWindowStore>,
    query: &BatchedRedactionWindowQuery,
) -> Vec<BatchedRedactionWindow> {
    query.apply(&store.list())
}
