use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding rollups by code.
pub trait BindingRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingRollup>;
    fn save(&self, item: &BindingRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingRollupStore {
    pub items: RefCell<BTreeMap<String, BindingRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingRollupStore for MemoryBindingRollupStore {
    fn find(&self, code: &str) -> Option<BindingRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingRollup) {
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

    fn list(&self) -> Vec<BindingRollup> {
        let mut out: Vec<BindingRollup> = Vec::new();
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

pub fn new_memory_binding_rollup_store() -> Rc<dyn BindingRollupStore> {
    Rc::new(MemoryBindingRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingRollupStore {
    pub inner: Rc<dyn BindingRollupStore>,
    pub cache: RefCell<cache::TtlCache<BindingRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingRollupStore for CachedBindingRollupStore {
    fn find(&self, code: &str) -> Option<BindingRollup> {
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

    fn save(&self, item: &BindingRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingRollup> {
        self.inner.list()
    }
}

pub fn new_cached_binding_rollup_store(
    inner: Rc<dyn BindingRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingRollupStore> {
    Rc::new(CachedBindingRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingRollup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingRollupQuery {
    pub source: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingRollupQuery {
    pub fn matches(&self, item: &BindingRollup) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingRollup]) -> Vec<BindingRollup> {
        let matched = collections::filter(items, |candidate: &BindingRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_rollups(store: Rc<dyn BindingRollupStore>, query: &BindingRollupQuery) -> Vec<BindingRollup> {
    query.apply(&store.list())
}
