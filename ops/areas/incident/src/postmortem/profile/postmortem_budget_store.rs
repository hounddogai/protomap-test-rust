use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps postmortem budgets by code.
pub trait PostmortemBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PostmortemBudget>;
    fn save(&self, item: &PostmortemBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PostmortemBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPostmortemBudgetStore {
    pub items: RefCell<BTreeMap<String, PostmortemBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl PostmortemBudgetStore for MemoryPostmortemBudgetStore {
    fn find(&self, code: &str) -> Option<PostmortemBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PostmortemBudget) {
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

    fn list(&self) -> Vec<PostmortemBudget> {
        let mut out: Vec<PostmortemBudget> = Vec::new();
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

pub fn new_memory_postmortem_budget_store() -> Rc<dyn PostmortemBudgetStore> {
    Rc::new(MemoryPostmortemBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPostmortemBudgetStore {
    pub inner: Rc<dyn PostmortemBudgetStore>,
    pub cache: RefCell<cache::TtlCache<PostmortemBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PostmortemBudgetStore for CachedPostmortemBudgetStore {
    fn find(&self, code: &str) -> Option<PostmortemBudget> {
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

    fn save(&self, item: &PostmortemBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PostmortemBudget> {
        self.inner.list()
    }
}

pub fn new_cached_postmortem_budget_store(
    inner: Rc<dyn PostmortemBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PostmortemBudgetStore> {
    Rc::new(CachedPostmortemBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PostmortemBudget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects postmortem budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBudgetQuery {
    pub unit: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PostmortemBudgetQuery {
    pub fn matches(&self, item: &PostmortemBudget) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PostmortemBudget]) -> Vec<PostmortemBudget> {
        let matched = collections::filter(items, |candidate: &PostmortemBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_postmortem_budgets(
    store: Rc<dyn PostmortemBudgetStore>,
    query: &PostmortemBudgetQuery,
) -> Vec<PostmortemBudget> {
    query.apply(&store.list())
}
