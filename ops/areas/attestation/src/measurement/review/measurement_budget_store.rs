use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement budgets by code.
pub trait MeasurementBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementBudget>;
    fn save(&self, item: &MeasurementBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementBudgetStore {
    pub items: RefCell<BTreeMap<String, MeasurementBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementBudgetStore for MemoryMeasurementBudgetStore {
    fn find(&self, code: &str) -> Option<MeasurementBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementBudget) {
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

    fn list(&self) -> Vec<MeasurementBudget> {
        let mut out: Vec<MeasurementBudget> = Vec::new();
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

pub fn new_memory_measurement_budget_store() -> Rc<dyn MeasurementBudgetStore> {
    Rc::new(MemoryMeasurementBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementBudgetStore {
    pub inner: Rc<dyn MeasurementBudgetStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementBudgetStore for CachedMeasurementBudgetStore {
    fn find(&self, code: &str) -> Option<MeasurementBudget> {
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

    fn save(&self, item: &MeasurementBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementBudget> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_budget_store(
    inner: Rc<dyn MeasurementBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementBudgetStore> {
    Rc::new(CachedMeasurementBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementBudget>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBudgetQuery {
    pub name: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementBudgetQuery {
    pub fn matches(&self, item: &MeasurementBudget) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[MeasurementBudget]) -> Vec<MeasurementBudget> {
        let matched = collections::filter(items, |candidate: &MeasurementBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_budgets(
    store: Rc<dyn MeasurementBudgetStore>,
    query: &MeasurementBudgetQuery,
) -> Vec<MeasurementBudget> {
    query.apply(&store.list())
}
