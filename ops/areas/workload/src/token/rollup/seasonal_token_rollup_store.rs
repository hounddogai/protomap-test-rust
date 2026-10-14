use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal token rollups by code.
pub trait SeasonalTokenRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalTokenRollup>;
    fn save(&self, item: &SeasonalTokenRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalTokenRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalTokenRollupStore {
    pub items: RefCell<BTreeMap<String, SeasonalTokenRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalTokenRollupStore for MemorySeasonalTokenRollupStore {
    fn find(&self, code: &str) -> Option<SeasonalTokenRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalTokenRollup) {
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

    fn list(&self) -> Vec<SeasonalTokenRollup> {
        let mut out: Vec<SeasonalTokenRollup> = Vec::new();
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

pub fn new_memory_seasonal_token_rollup_store() -> Rc<dyn SeasonalTokenRollupStore> {
    Rc::new(MemorySeasonalTokenRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalTokenRollupStore {
    pub inner: Rc<dyn SeasonalTokenRollupStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalTokenRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalTokenRollupStore for CachedSeasonalTokenRollupStore {
    fn find(&self, code: &str) -> Option<SeasonalTokenRollup> {
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

    fn save(&self, item: &SeasonalTokenRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalTokenRollup> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_token_rollup_store(
    inner: Rc<dyn SeasonalTokenRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalTokenRollupStore> {
    Rc::new(CachedSeasonalTokenRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalTokenRollup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal token rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenRollupQuery {
    pub name: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalTokenRollupQuery {
    pub fn matches(&self, item: &SeasonalTokenRollup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalTokenRollup]) -> Vec<SeasonalTokenRollup> {
        let matched = collections::filter(items, |candidate: &SeasonalTokenRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_token_rollups(
    store: Rc<dyn SeasonalTokenRollupStore>,
    query: &SeasonalTokenRollupQuery,
) -> Vec<SeasonalTokenRollup> {
    query.apply(&store.list())
}
