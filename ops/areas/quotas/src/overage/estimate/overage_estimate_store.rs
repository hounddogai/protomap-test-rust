use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps overage estimates by code.
pub trait OverageEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<OverageEstimate>;
    fn save(&self, item: &OverageEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<OverageEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryOverageEstimateStore {
    pub items: RefCell<BTreeMap<String, OverageEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl OverageEstimateStore for MemoryOverageEstimateStore {
    fn find(&self, code: &str) -> Option<OverageEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &OverageEstimate) {
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

    fn list(&self) -> Vec<OverageEstimate> {
        let mut out: Vec<OverageEstimate> = Vec::new();
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

pub fn new_memory_overage_estimate_store() -> Rc<dyn OverageEstimateStore> {
    Rc::new(MemoryOverageEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedOverageEstimateStore {
    pub inner: Rc<dyn OverageEstimateStore>,
    pub cache: RefCell<cache::TtlCache<OverageEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl OverageEstimateStore for CachedOverageEstimateStore {
    fn find(&self, code: &str) -> Option<OverageEstimate> {
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

    fn save(&self, item: &OverageEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<OverageEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_overage_estimate_store(
    inner: Rc<dyn OverageEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn OverageEstimateStore> {
    Rc::new(CachedOverageEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<OverageEstimate>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects overage estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageEstimateQuery {
    pub country: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl OverageEstimateQuery {
    pub fn matches(&self, item: &OverageEstimate) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[OverageEstimate]) -> Vec<OverageEstimate> {
        let matched = collections::filter(items, |candidate: &OverageEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_overage_estimates(
    store: Rc<dyn OverageEstimateStore>,
    query: &OverageEstimateQuery,
) -> Vec<OverageEstimate> {
    query.apply(&store.list())
}
