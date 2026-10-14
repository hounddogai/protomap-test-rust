use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled rollout budgets by code.
pub trait ScheduledRolloutBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledRolloutBudget>;
    fn save(&self, item: &ScheduledRolloutBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledRolloutBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledRolloutBudgetStore {
    pub items: RefCell<BTreeMap<String, ScheduledRolloutBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledRolloutBudgetStore for MemoryScheduledRolloutBudgetStore {
    fn find(&self, code: &str) -> Option<ScheduledRolloutBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledRolloutBudget) {
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

    fn list(&self) -> Vec<ScheduledRolloutBudget> {
        let mut out: Vec<ScheduledRolloutBudget> = Vec::new();
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

pub fn new_memory_scheduled_rollout_budget_store() -> Rc<dyn ScheduledRolloutBudgetStore> {
    Rc::new(MemoryScheduledRolloutBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledRolloutBudgetStore {
    pub inner: Rc<dyn ScheduledRolloutBudgetStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledRolloutBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledRolloutBudgetStore for CachedScheduledRolloutBudgetStore {
    fn find(&self, code: &str) -> Option<ScheduledRolloutBudget> {
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

    fn save(&self, item: &ScheduledRolloutBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledRolloutBudget> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_rollout_budget_store(
    inner: Rc<dyn ScheduledRolloutBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledRolloutBudgetStore> {
    Rc::new(CachedScheduledRolloutBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledRolloutBudget>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled rollout budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutBudgetQuery {
    pub source: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledRolloutBudgetQuery {
    pub fn matches(&self, item: &ScheduledRolloutBudget) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledRolloutBudget]) -> Vec<ScheduledRolloutBudget> {
        let matched = collections::filter(items, |candidate: &ScheduledRolloutBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_rollout_budgets(
    store: Rc<dyn ScheduledRolloutBudgetStore>,
    query: &ScheduledRolloutBudgetQuery,
) -> Vec<ScheduledRolloutBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledRolloutBudgetStore> = new_memory_scheduled_rollout_budget_store();
        let item = new_scheduled_rollout_budget("code-680", "label-590");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
