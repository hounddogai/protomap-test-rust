use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy grant rollups by code.
pub trait LegacyGrantRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyGrantRollup>;
    fn save(&self, item: &LegacyGrantRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyGrantRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyGrantRollupStore {
    pub items: RefCell<BTreeMap<String, LegacyGrantRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyGrantRollupStore for MemoryLegacyGrantRollupStore {
    fn find(&self, code: &str) -> Option<LegacyGrantRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyGrantRollup) {
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

    fn list(&self) -> Vec<LegacyGrantRollup> {
        let mut out: Vec<LegacyGrantRollup> = Vec::new();
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

pub fn new_memory_legacy_grant_rollup_store() -> Rc<dyn LegacyGrantRollupStore> {
    Rc::new(MemoryLegacyGrantRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyGrantRollupStore {
    pub inner: Rc<dyn LegacyGrantRollupStore>,
    pub cache: RefCell<cache::TtlCache<LegacyGrantRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyGrantRollupStore for CachedLegacyGrantRollupStore {
    fn find(&self, code: &str) -> Option<LegacyGrantRollup> {
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

    fn save(&self, item: &LegacyGrantRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyGrantRollup> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_grant_rollup_store(
    inner: Rc<dyn LegacyGrantRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyGrantRollupStore> {
    Rc::new(CachedLegacyGrantRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyGrantRollup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy grant rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantRollupQuery {
    pub unit: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyGrantRollupQuery {
    pub fn matches(&self, item: &LegacyGrantRollup) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[LegacyGrantRollup]) -> Vec<LegacyGrantRollup> {
        let matched = collections::filter(items, |candidate: &LegacyGrantRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_grant_rollups(
    store: Rc<dyn LegacyGrantRollupStore>,
    query: &LegacyGrantRollupQuery,
) -> Vec<LegacyGrantRollup> {
    query.apply(&store.list())
}
