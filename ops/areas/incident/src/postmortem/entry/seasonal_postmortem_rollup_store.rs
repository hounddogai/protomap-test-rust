use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal postmortem rollups by code.
pub trait SeasonalPostmortemRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalPostmortemRollup>;
    fn save(&self, item: &SeasonalPostmortemRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalPostmortemRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalPostmortemRollupStore {
    pub items: RefCell<BTreeMap<String, SeasonalPostmortemRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalPostmortemRollupStore for MemorySeasonalPostmortemRollupStore {
    fn find(&self, code: &str) -> Option<SeasonalPostmortemRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalPostmortemRollup) {
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

    fn list(&self) -> Vec<SeasonalPostmortemRollup> {
        let mut out: Vec<SeasonalPostmortemRollup> = Vec::new();
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

pub fn new_memory_seasonal_postmortem_rollup_store() -> Rc<dyn SeasonalPostmortemRollupStore> {
    Rc::new(MemorySeasonalPostmortemRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalPostmortemRollupStore {
    pub inner: Rc<dyn SeasonalPostmortemRollupStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalPostmortemRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalPostmortemRollupStore for CachedSeasonalPostmortemRollupStore {
    fn find(&self, code: &str) -> Option<SeasonalPostmortemRollup> {
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

    fn save(&self, item: &SeasonalPostmortemRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalPostmortemRollup> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_postmortem_rollup_store(
    inner: Rc<dyn SeasonalPostmortemRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalPostmortemRollupStore> {
    Rc::new(CachedSeasonalPostmortemRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalPostmortemRollup>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal postmortem rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemRollupQuery {
    pub title: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalPostmortemRollupQuery {
    pub fn matches(&self, item: &SeasonalPostmortemRollup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalPostmortemRollup]) -> Vec<SeasonalPostmortemRollup> {
        let matched = collections::filter(items, |candidate: &SeasonalPostmortemRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_postmortem_rollups(
    store: Rc<dyn SeasonalPostmortemRollupStore>,
    query: &SeasonalPostmortemRollupQuery,
) -> Vec<SeasonalPostmortemRollup> {
    query.apply(&store.list())
}
