use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log budgets by code.
pub trait FlowLogBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogBudget>;
    fn save(&self, item: &FlowLogBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogBudgetStore {
    pub items: RefCell<BTreeMap<String, FlowLogBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogBudgetStore for MemoryFlowLogBudgetStore {
    fn find(&self, code: &str) -> Option<FlowLogBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogBudget) {
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

    fn list(&self) -> Vec<FlowLogBudget> {
        let mut out: Vec<FlowLogBudget> = Vec::new();
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

pub fn new_memory_flow_log_budget_store() -> Rc<dyn FlowLogBudgetStore> {
    Rc::new(MemoryFlowLogBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogBudgetStore {
    pub inner: Rc<dyn FlowLogBudgetStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogBudgetStore for CachedFlowLogBudgetStore {
    fn find(&self, code: &str) -> Option<FlowLogBudget> {
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

    fn save(&self, item: &FlowLogBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogBudget> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_budget_store(
    inner: Rc<dyn FlowLogBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogBudgetStore> {
    Rc::new(CachedFlowLogBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogBudget>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBudgetQuery {
    pub country: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogBudgetQuery {
    pub fn matches(&self, item: &FlowLogBudget) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogBudget]) -> Vec<FlowLogBudget> {
        let matched = collections::filter(items, |candidate: &FlowLogBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_budgets(store: Rc<dyn FlowLogBudgetStore>, query: &FlowLogBudgetQuery) -> Vec<FlowLogBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FlowLogBudgetStore> = new_memory_flow_log_budget_store();
        let item = new_flow_log_budget("code-370", "name-884");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
