use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps failover targets by code.
pub trait FailoverTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FailoverTarget>;
    fn save(&self, item: &FailoverTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FailoverTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFailoverTargetStore {
    pub items: RefCell<BTreeMap<String, FailoverTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl FailoverTargetStore for MemoryFailoverTargetStore {
    fn find(&self, code: &str) -> Option<FailoverTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FailoverTarget) {
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

    fn list(&self) -> Vec<FailoverTarget> {
        let mut out: Vec<FailoverTarget> = Vec::new();
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

pub fn new_memory_failover_target_store() -> Rc<dyn FailoverTargetStore> {
    Rc::new(MemoryFailoverTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFailoverTargetStore {
    pub inner: Rc<dyn FailoverTargetStore>,
    pub cache: RefCell<cache::TtlCache<FailoverTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FailoverTargetStore for CachedFailoverTargetStore {
    fn find(&self, code: &str) -> Option<FailoverTarget> {
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

    fn save(&self, item: &FailoverTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FailoverTarget> {
        self.inner.list()
    }
}

pub fn new_cached_failover_target_store(
    inner: Rc<dyn FailoverTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FailoverTargetStore> {
    Rc::new(CachedFailoverTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FailoverTarget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects failover targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverTargetQuery {
    pub source: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FailoverTargetQuery {
    pub fn matches(&self, item: &FailoverTarget) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FailoverTarget]) -> Vec<FailoverTarget> {
        let matched = collections::filter(items, |candidate: &FailoverTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_failover_targets(store: Rc<dyn FailoverTargetStore>, query: &FailoverTargetQuery) -> Vec<FailoverTarget> {
    query.apply(&store.list())
}
