use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive query adjustments by code.
pub trait AdaptiveQueryAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveQueryAdjustment>;
    fn save(&self, item: &AdaptiveQueryAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveQueryAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveQueryAdjustmentStore {
    pub items: RefCell<BTreeMap<String, AdaptiveQueryAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveQueryAdjustmentStore for MemoryAdaptiveQueryAdjustmentStore {
    fn find(&self, code: &str) -> Option<AdaptiveQueryAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveQueryAdjustment) {
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

    fn list(&self) -> Vec<AdaptiveQueryAdjustment> {
        let mut out: Vec<AdaptiveQueryAdjustment> = Vec::new();
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

pub fn new_memory_adaptive_query_adjustment_store() -> Rc<dyn AdaptiveQueryAdjustmentStore> {
    Rc::new(MemoryAdaptiveQueryAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveQueryAdjustmentStore {
    pub inner: Rc<dyn AdaptiveQueryAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveQueryAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveQueryAdjustmentStore for CachedAdaptiveQueryAdjustmentStore {
    fn find(&self, code: &str) -> Option<AdaptiveQueryAdjustment> {
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

    fn save(&self, item: &AdaptiveQueryAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveQueryAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_query_adjustment_store(
    inner: Rc<dyn AdaptiveQueryAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveQueryAdjustmentStore> {
    Rc::new(CachedAdaptiveQueryAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveQueryAdjustment>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive query adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustmentQuery {
    pub title: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveQueryAdjustmentQuery {
    pub fn matches(&self, item: &AdaptiveQueryAdjustment) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveQueryAdjustment]) -> Vec<AdaptiveQueryAdjustment> {
        let matched = collections::filter(items, |candidate: &AdaptiveQueryAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_query_adjustments(
    store: Rc<dyn AdaptiveQueryAdjustmentStore>,
    query: &AdaptiveQueryAdjustmentQuery,
) -> Vec<AdaptiveQueryAdjustment> {
    query.apply(&store.list())
}
