use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional rollback targets by code.
pub trait RegionalRollbackTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalRollbackTarget>;
    fn save(&self, item: &RegionalRollbackTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalRollbackTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalRollbackTargetStore {
    pub items: RefCell<BTreeMap<String, RegionalRollbackTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalRollbackTargetStore for MemoryRegionalRollbackTargetStore {
    fn find(&self, code: &str) -> Option<RegionalRollbackTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalRollbackTarget) {
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

    fn list(&self) -> Vec<RegionalRollbackTarget> {
        let mut out: Vec<RegionalRollbackTarget> = Vec::new();
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

pub fn new_memory_regional_rollback_target_store() -> Rc<dyn RegionalRollbackTargetStore> {
    Rc::new(MemoryRegionalRollbackTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalRollbackTargetStore {
    pub inner: Rc<dyn RegionalRollbackTargetStore>,
    pub cache: RefCell<cache::TtlCache<RegionalRollbackTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalRollbackTargetStore for CachedRegionalRollbackTargetStore {
    fn find(&self, code: &str) -> Option<RegionalRollbackTarget> {
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

    fn save(&self, item: &RegionalRollbackTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalRollbackTarget> {
        self.inner.list()
    }
}

pub fn new_cached_regional_rollback_target_store(
    inner: Rc<dyn RegionalRollbackTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalRollbackTargetStore> {
    Rc::new(CachedRegionalRollbackTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalRollbackTarget>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional rollback targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRollbackTargetQuery {
    pub scope: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalRollbackTargetQuery {
    pub fn matches(&self, item: &RegionalRollbackTarget) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalRollbackTarget]) -> Vec<RegionalRollbackTarget> {
        let matched = collections::filter(items, |candidate: &RegionalRollbackTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_rollback_targets(
    store: Rc<dyn RegionalRollbackTargetStore>,
    query: &RegionalRollbackTargetQuery,
) -> Vec<RegionalRollbackTarget> {
    query.apply(&store.list())
}
