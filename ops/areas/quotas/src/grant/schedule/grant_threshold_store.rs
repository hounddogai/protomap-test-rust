use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps grant thresholds by code.
pub trait GrantThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GrantThreshold>;
    fn save(&self, item: &GrantThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GrantThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGrantThresholdStore {
    pub items: RefCell<BTreeMap<String, GrantThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl GrantThresholdStore for MemoryGrantThresholdStore {
    fn find(&self, code: &str) -> Option<GrantThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GrantThreshold) {
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

    fn list(&self) -> Vec<GrantThreshold> {
        let mut out: Vec<GrantThreshold> = Vec::new();
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

pub fn new_memory_grant_threshold_store() -> Rc<dyn GrantThresholdStore> {
    Rc::new(MemoryGrantThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGrantThresholdStore {
    pub inner: Rc<dyn GrantThresholdStore>,
    pub cache: RefCell<cache::TtlCache<GrantThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GrantThresholdStore for CachedGrantThresholdStore {
    fn find(&self, code: &str) -> Option<GrantThreshold> {
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

    fn save(&self, item: &GrantThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GrantThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_grant_threshold_store(
    inner: Rc<dyn GrantThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GrantThresholdStore> {
    Rc::new(CachedGrantThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GrantThreshold>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects grant thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantThresholdQuery {
    pub category: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GrantThresholdQuery {
    pub fn matches(&self, item: &GrantThreshold) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GrantThreshold]) -> Vec<GrantThreshold> {
        let matched = collections::filter(items, |candidate: &GrantThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_grant_thresholds(store: Rc<dyn GrantThresholdStore>, query: &GrantThresholdQuery) -> Vec<GrantThreshold> {
    query.apply(&store.list())
}
