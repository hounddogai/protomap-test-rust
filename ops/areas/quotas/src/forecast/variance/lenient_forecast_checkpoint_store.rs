use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient forecast checkpoints by code.
pub trait LenientForecastCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientForecastCheckpoint>;
    fn save(&self, item: &LenientForecastCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientForecastCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientForecastCheckpointStore {
    pub items: RefCell<BTreeMap<String, LenientForecastCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientForecastCheckpointStore for MemoryLenientForecastCheckpointStore {
    fn find(&self, code: &str) -> Option<LenientForecastCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientForecastCheckpoint) {
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

    fn list(&self) -> Vec<LenientForecastCheckpoint> {
        let mut out: Vec<LenientForecastCheckpoint> = Vec::new();
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

pub fn new_memory_lenient_forecast_checkpoint_store() -> Rc<dyn LenientForecastCheckpointStore> {
    Rc::new(MemoryLenientForecastCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientForecastCheckpointStore {
    pub inner: Rc<dyn LenientForecastCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<LenientForecastCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientForecastCheckpointStore for CachedLenientForecastCheckpointStore {
    fn find(&self, code: &str) -> Option<LenientForecastCheckpoint> {
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

    fn save(&self, item: &LenientForecastCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientForecastCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_forecast_checkpoint_store(
    inner: Rc<dyn LenientForecastCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientForecastCheckpointStore> {
    Rc::new(CachedLenientForecastCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientForecastCheckpoint>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient forecast checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastCheckpointQuery {
    pub label: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientForecastCheckpointQuery {
    pub fn matches(&self, item: &LenientForecastCheckpoint) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientForecastCheckpoint]) -> Vec<LenientForecastCheckpoint> {
        let matched = collections::filter(items, |candidate: &LenientForecastCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_forecast_checkpoints(
    store: Rc<dyn LenientForecastCheckpointStore>,
    query: &LenientForecastCheckpointQuery,
) -> Vec<LenientForecastCheckpoint> {
    query.apply(&store.list())
}
