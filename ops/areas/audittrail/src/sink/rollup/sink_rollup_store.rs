use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink rollups by code.
pub trait SinkRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkRollup>;
    fn save(&self, item: &SinkRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkRollupStore {
    pub items: RefCell<BTreeMap<String, SinkRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkRollupStore for MemorySinkRollupStore {
    fn find(&self, code: &str) -> Option<SinkRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkRollup) {
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

    fn list(&self) -> Vec<SinkRollup> {
        let mut out: Vec<SinkRollup> = Vec::new();
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

pub fn new_memory_sink_rollup_store() -> Rc<dyn SinkRollupStore> {
    Rc::new(MemorySinkRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkRollupStore {
    pub inner: Rc<dyn SinkRollupStore>,
    pub cache: RefCell<cache::TtlCache<SinkRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkRollupStore for CachedSinkRollupStore {
    fn find(&self, code: &str) -> Option<SinkRollup> {
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

    fn save(&self, item: &SinkRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkRollup> {
        self.inner.list()
    }
}

pub fn new_cached_sink_rollup_store(
    inner: Rc<dyn SinkRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkRollupStore> {
    Rc::new(CachedSinkRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkRollup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRollupQuery {
    pub scope: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkRollupQuery {
    pub fn matches(&self, item: &SinkRollup) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SinkRollup]) -> Vec<SinkRollup> {
        let matched = collections::filter(items, |candidate: &SinkRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_rollups(store: Rc<dyn SinkRollupStore>, query: &SinkRollupQuery) -> Vec<SinkRollup> {
    query.apply(&store.list())
}
