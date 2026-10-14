use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps canary overrides by code.
pub trait CanaryOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CanaryOverride>;
    fn save(&self, item: &CanaryOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CanaryOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCanaryOverrideStore {
    pub items: RefCell<BTreeMap<String, CanaryOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl CanaryOverrideStore for MemoryCanaryOverrideStore {
    fn find(&self, code: &str) -> Option<CanaryOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CanaryOverride) {
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

    fn list(&self) -> Vec<CanaryOverride> {
        let mut out: Vec<CanaryOverride> = Vec::new();
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

pub fn new_memory_canary_override_store() -> Rc<dyn CanaryOverrideStore> {
    Rc::new(MemoryCanaryOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCanaryOverrideStore {
    pub inner: Rc<dyn CanaryOverrideStore>,
    pub cache: RefCell<cache::TtlCache<CanaryOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CanaryOverrideStore for CachedCanaryOverrideStore {
    fn find(&self, code: &str) -> Option<CanaryOverride> {
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

    fn save(&self, item: &CanaryOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CanaryOverride> {
        self.inner.list()
    }
}

pub fn new_cached_canary_override_store(
    inner: Rc<dyn CanaryOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CanaryOverrideStore> {
    Rc::new(CachedCanaryOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CanaryOverride>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects canary overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryOverrideQuery {
    pub title: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CanaryOverrideQuery {
    pub fn matches(&self, item: &CanaryOverride) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[CanaryOverride]) -> Vec<CanaryOverride> {
        let matched = collections::filter(items, |candidate: &CanaryOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_canary_overrides(store: Rc<dyn CanaryOverrideStore>, query: &CanaryOverrideQuery) -> Vec<CanaryOverride> {
    query.apply(&store.list())
}
