use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink budgets by code.
pub trait SinkBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkBudget>;
    fn save(&self, item: &SinkBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkBudgetStore {
    pub items: RefCell<BTreeMap<String, SinkBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkBudgetStore for MemorySinkBudgetStore {
    fn find(&self, code: &str) -> Option<SinkBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkBudget) {
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

    fn list(&self) -> Vec<SinkBudget> {
        let mut out: Vec<SinkBudget> = Vec::new();
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

pub fn new_memory_sink_budget_store() -> Rc<dyn SinkBudgetStore> {
    Rc::new(MemorySinkBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkBudgetStore {
    pub inner: Rc<dyn SinkBudgetStore>,
    pub cache: RefCell<cache::TtlCache<SinkBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkBudgetStore for CachedSinkBudgetStore {
    fn find(&self, code: &str) -> Option<SinkBudget> {
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

    fn save(&self, item: &SinkBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkBudget> {
        self.inner.list()
    }
}

pub fn new_cached_sink_budget_store(
    inner: Rc<dyn SinkBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkBudgetStore> {
    Rc::new(CachedSinkBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkBudget>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudgetQuery {
    pub category: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkBudgetQuery {
    pub fn matches(&self, item: &SinkBudget) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SinkBudget]) -> Vec<SinkBudget> {
        let matched = collections::filter(items, |candidate: &SinkBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_budgets(store: Rc<dyn SinkBudgetStore>, query: &SinkBudgetQuery) -> Vec<SinkBudget> {
    query.apply(&store.list())
}
