use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual limit budgets by code.
pub trait ManualLimitBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualLimitBudget>;
    fn save(&self, item: &ManualLimitBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualLimitBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualLimitBudgetStore {
    pub items: RefCell<BTreeMap<String, ManualLimitBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualLimitBudgetStore for MemoryManualLimitBudgetStore {
    fn find(&self, code: &str) -> Option<ManualLimitBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualLimitBudget) {
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

    fn list(&self) -> Vec<ManualLimitBudget> {
        let mut out: Vec<ManualLimitBudget> = Vec::new();
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

pub fn new_memory_manual_limit_budget_store() -> Rc<dyn ManualLimitBudgetStore> {
    Rc::new(MemoryManualLimitBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualLimitBudgetStore {
    pub inner: Rc<dyn ManualLimitBudgetStore>,
    pub cache: RefCell<cache::TtlCache<ManualLimitBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualLimitBudgetStore for CachedManualLimitBudgetStore {
    fn find(&self, code: &str) -> Option<ManualLimitBudget> {
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

    fn save(&self, item: &ManualLimitBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualLimitBudget> {
        self.inner.list()
    }
}

pub fn new_cached_manual_limit_budget_store(
    inner: Rc<dyn ManualLimitBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualLimitBudgetStore> {
    Rc::new(CachedManualLimitBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualLimitBudget>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual limit budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitBudgetQuery {
    pub country: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualLimitBudgetQuery {
    pub fn matches(&self, item: &ManualLimitBudget) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualLimitBudget]) -> Vec<ManualLimitBudget> {
        let matched = collections::filter(items, |candidate: &ManualLimitBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_limit_budgets(
    store: Rc<dyn ManualLimitBudgetStore>,
    query: &ManualLimitBudgetQuery,
) -> Vec<ManualLimitBudget> {
    query.apply(&store.list())
}
