use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps allowlist targets by code.
pub trait AllowlistTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AllowlistTarget>;
    fn save(&self, item: &AllowlistTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AllowlistTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAllowlistTargetStore {
    pub items: RefCell<BTreeMap<String, AllowlistTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl AllowlistTargetStore for MemoryAllowlistTargetStore {
    fn find(&self, code: &str) -> Option<AllowlistTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AllowlistTarget) {
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

    fn list(&self) -> Vec<AllowlistTarget> {
        let mut out: Vec<AllowlistTarget> = Vec::new();
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

pub fn new_memory_allowlist_target_store() -> Rc<dyn AllowlistTargetStore> {
    Rc::new(MemoryAllowlistTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAllowlistTargetStore {
    pub inner: Rc<dyn AllowlistTargetStore>,
    pub cache: RefCell<cache::TtlCache<AllowlistTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AllowlistTargetStore for CachedAllowlistTargetStore {
    fn find(&self, code: &str) -> Option<AllowlistTarget> {
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

    fn save(&self, item: &AllowlistTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AllowlistTarget> {
        self.inner.list()
    }
}

pub fn new_cached_allowlist_target_store(
    inner: Rc<dyn AllowlistTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AllowlistTargetStore> {
    Rc::new(CachedAllowlistTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AllowlistTarget>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects allowlist targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTargetQuery {
    pub label: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AllowlistTargetQuery {
    pub fn matches(&self, item: &AllowlistTarget) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[AllowlistTarget]) -> Vec<AllowlistTarget> {
        let matched = collections::filter(items, |candidate: &AllowlistTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_allowlist_targets(
    store: Rc<dyn AllowlistTargetStore>,
    query: &AllowlistTargetQuery,
) -> Vec<AllowlistTarget> {
    query.apply(&store.list())
}
