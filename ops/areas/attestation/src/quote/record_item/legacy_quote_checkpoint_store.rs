use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy quote checkpoints by code.
pub trait LegacyQuoteCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyQuoteCheckpoint>;
    fn save(&self, item: &LegacyQuoteCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyQuoteCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyQuoteCheckpointStore {
    pub items: RefCell<BTreeMap<String, LegacyQuoteCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyQuoteCheckpointStore for MemoryLegacyQuoteCheckpointStore {
    fn find(&self, code: &str) -> Option<LegacyQuoteCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyQuoteCheckpoint) {
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

    fn list(&self) -> Vec<LegacyQuoteCheckpoint> {
        let mut out: Vec<LegacyQuoteCheckpoint> = Vec::new();
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

pub fn new_memory_legacy_quote_checkpoint_store() -> Rc<dyn LegacyQuoteCheckpointStore> {
    Rc::new(MemoryLegacyQuoteCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyQuoteCheckpointStore {
    pub inner: Rc<dyn LegacyQuoteCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<LegacyQuoteCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyQuoteCheckpointStore for CachedLegacyQuoteCheckpointStore {
    fn find(&self, code: &str) -> Option<LegacyQuoteCheckpoint> {
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

    fn save(&self, item: &LegacyQuoteCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyQuoteCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_quote_checkpoint_store(
    inner: Rc<dyn LegacyQuoteCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyQuoteCheckpointStore> {
    Rc::new(CachedLegacyQuoteCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyQuoteCheckpoint>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy quote checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteCheckpointQuery {
    pub region: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyQuoteCheckpointQuery {
    pub fn matches(&self, item: &LegacyQuoteCheckpoint) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyQuoteCheckpoint]) -> Vec<LegacyQuoteCheckpoint> {
        let matched = collections::filter(items, |candidate: &LegacyQuoteCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_quote_checkpoints(
    store: Rc<dyn LegacyQuoteCheckpointStore>,
    query: &LegacyQuoteCheckpointQuery,
) -> Vec<LegacyQuoteCheckpoint> {
    query.apply(&store.list())
}
