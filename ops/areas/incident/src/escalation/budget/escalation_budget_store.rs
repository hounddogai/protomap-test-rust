use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps escalation budgets by code.
pub trait EscalationBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EscalationBudget>;
    fn save(&self, item: &EscalationBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EscalationBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEscalationBudgetStore {
    pub items: RefCell<BTreeMap<String, EscalationBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl EscalationBudgetStore for MemoryEscalationBudgetStore {
    fn find(&self, code: &str) -> Option<EscalationBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EscalationBudget) {
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

    fn list(&self) -> Vec<EscalationBudget> {
        let mut out: Vec<EscalationBudget> = Vec::new();
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

pub fn new_memory_escalation_budget_store() -> Rc<dyn EscalationBudgetStore> {
    Rc::new(MemoryEscalationBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEscalationBudgetStore {
    pub inner: Rc<dyn EscalationBudgetStore>,
    pub cache: RefCell<cache::TtlCache<EscalationBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EscalationBudgetStore for CachedEscalationBudgetStore {
    fn find(&self, code: &str) -> Option<EscalationBudget> {
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

    fn save(&self, item: &EscalationBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EscalationBudget> {
        self.inner.list()
    }
}

pub fn new_cached_escalation_budget_store(
    inner: Rc<dyn EscalationBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EscalationBudgetStore> {
    Rc::new(CachedEscalationBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EscalationBudget>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects escalation budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudgetQuery {
    pub title: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EscalationBudgetQuery {
    pub fn matches(&self, item: &EscalationBudget) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[EscalationBudget]) -> Vec<EscalationBudget> {
        let matched = collections::filter(items, |candidate: &EscalationBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_escalation_budgets(
    store: Rc<dyn EscalationBudgetStore>,
    query: &EscalationBudgetQuery,
) -> Vec<EscalationBudget> {
    query.apply(&store.list())
}
