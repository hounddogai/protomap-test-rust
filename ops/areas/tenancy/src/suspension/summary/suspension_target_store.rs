use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension targets by code.
pub trait SuspensionTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionTarget>;
    fn save(&self, item: &SuspensionTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionTargetStore {
    pub items: RefCell<BTreeMap<String, SuspensionTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionTargetStore for MemorySuspensionTargetStore {
    fn find(&self, code: &str) -> Option<SuspensionTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionTarget) {
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

    fn list(&self) -> Vec<SuspensionTarget> {
        let mut out: Vec<SuspensionTarget> = Vec::new();
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

pub fn new_memory_suspension_target_store() -> Rc<dyn SuspensionTargetStore> {
    Rc::new(MemorySuspensionTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionTargetStore {
    pub inner: Rc<dyn SuspensionTargetStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionTargetStore for CachedSuspensionTargetStore {
    fn find(&self, code: &str) -> Option<SuspensionTarget> {
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

    fn save(&self, item: &SuspensionTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionTarget> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_target_store(
    inner: Rc<dyn SuspensionTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionTargetStore> {
    Rc::new(CachedSuspensionTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionTarget>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTargetQuery {
    pub namespace: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionTargetQuery {
    pub fn matches(&self, item: &SuspensionTarget) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SuspensionTarget]) -> Vec<SuspensionTarget> {
        let matched = collections::filter(items, |candidate: &SuspensionTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_targets(
    store: Rc<dyn SuspensionTargetStore>,
    query: &SuspensionTargetQuery,
) -> Vec<SuspensionTarget> {
    query.apply(&store.list())
}
