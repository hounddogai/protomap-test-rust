use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global page variances by code.
pub trait GlobalPageVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalPageVariance>;
    fn save(&self, item: &GlobalPageVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalPageVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalPageVarianceStore {
    pub items: RefCell<BTreeMap<String, GlobalPageVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalPageVarianceStore for MemoryGlobalPageVarianceStore {
    fn find(&self, code: &str) -> Option<GlobalPageVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalPageVariance) {
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

    fn list(&self) -> Vec<GlobalPageVariance> {
        let mut out: Vec<GlobalPageVariance> = Vec::new();
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

pub fn new_memory_global_page_variance_store() -> Rc<dyn GlobalPageVarianceStore> {
    Rc::new(MemoryGlobalPageVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalPageVarianceStore {
    pub inner: Rc<dyn GlobalPageVarianceStore>,
    pub cache: RefCell<cache::TtlCache<GlobalPageVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalPageVarianceStore for CachedGlobalPageVarianceStore {
    fn find(&self, code: &str) -> Option<GlobalPageVariance> {
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

    fn save(&self, item: &GlobalPageVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalPageVariance> {
        self.inner.list()
    }
}

pub fn new_cached_global_page_variance_store(
    inner: Rc<dyn GlobalPageVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalPageVarianceStore> {
    Rc::new(CachedGlobalPageVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalPageVariance>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global page variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageVarianceQuery {
    pub name: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalPageVarianceQuery {
    pub fn matches(&self, item: &GlobalPageVariance) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[GlobalPageVariance]) -> Vec<GlobalPageVariance> {
        let matched = collections::filter(items, |candidate: &GlobalPageVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_page_variances(
    store: Rc<dyn GlobalPageVarianceStore>,
    query: &GlobalPageVarianceQuery,
) -> Vec<GlobalPageVariance> {
    query.apply(&store.list())
}
