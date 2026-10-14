use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding thresholds by code.
pub trait BindingThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingThreshold>;
    fn save(&self, item: &BindingThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingThresholdStore {
    pub items: RefCell<BTreeMap<String, BindingThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingThresholdStore for MemoryBindingThresholdStore {
    fn find(&self, code: &str) -> Option<BindingThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingThreshold) {
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

    fn list(&self) -> Vec<BindingThreshold> {
        let mut out: Vec<BindingThreshold> = Vec::new();
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

pub fn new_memory_binding_threshold_store() -> Rc<dyn BindingThresholdStore> {
    Rc::new(MemoryBindingThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingThresholdStore {
    pub inner: Rc<dyn BindingThresholdStore>,
    pub cache: RefCell<cache::TtlCache<BindingThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingThresholdStore for CachedBindingThresholdStore {
    fn find(&self, code: &str) -> Option<BindingThreshold> {
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

    fn save(&self, item: &BindingThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_binding_threshold_store(
    inner: Rc<dyn BindingThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingThresholdStore> {
    Rc::new(CachedBindingThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingThreshold>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingThresholdQuery {
    pub source: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingThresholdQuery {
    pub fn matches(&self, item: &BindingThreshold) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingThreshold]) -> Vec<BindingThreshold> {
        let matched = collections::filter(items, |candidate: &BindingThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_thresholds(
    store: Rc<dyn BindingThresholdStore>,
    query: &BindingThresholdQuery,
) -> Vec<BindingThreshold> {
    query.apply(&store.list())
}
