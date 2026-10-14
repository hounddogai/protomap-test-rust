use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback rollups by code.
pub trait RollbackRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackRollup>;
    fn save(&self, item: &RollbackRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackRollupStore {
    pub items: RefCell<BTreeMap<String, RollbackRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackRollupStore for MemoryRollbackRollupStore {
    fn find(&self, code: &str) -> Option<RollbackRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackRollup) {
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

    fn list(&self) -> Vec<RollbackRollup> {
        let mut out: Vec<RollbackRollup> = Vec::new();
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

pub fn new_memory_rollback_rollup_store() -> Rc<dyn RollbackRollupStore> {
    Rc::new(MemoryRollbackRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackRollupStore {
    pub inner: Rc<dyn RollbackRollupStore>,
    pub cache: RefCell<cache::TtlCache<RollbackRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackRollupStore for CachedRollbackRollupStore {
    fn find(&self, code: &str) -> Option<RollbackRollup> {
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

    fn save(&self, item: &RollbackRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackRollup> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_rollup_store(
    inner: Rc<dyn RollbackRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackRollupStore> {
    Rc::new(CachedRollbackRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackRollup>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRollupQuery {
    pub locale: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackRollupQuery {
    pub fn matches(&self, item: &RollbackRollup) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollbackRollup]) -> Vec<RollbackRollup> {
        let matched = collections::filter(items, |candidate: &RollbackRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_rollups(store: Rc<dyn RollbackRollupStore>, query: &RollbackRollupQuery) -> Vec<RollbackRollup> {
    query.apply(&store.list())
}
