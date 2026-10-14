use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query budgets by code.
pub trait QueryBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryBudget>;
    fn save(&self, item: &QueryBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryBudgetStore {
    pub items: RefCell<BTreeMap<String, QueryBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryBudgetStore for MemoryQueryBudgetStore {
    fn find(&self, code: &str) -> Option<QueryBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryBudget) {
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

    fn list(&self) -> Vec<QueryBudget> {
        let mut out: Vec<QueryBudget> = Vec::new();
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

pub fn new_memory_query_budget_store() -> Rc<dyn QueryBudgetStore> {
    Rc::new(MemoryQueryBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryBudgetStore {
    pub inner: Rc<dyn QueryBudgetStore>,
    pub cache: RefCell<cache::TtlCache<QueryBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryBudgetStore for CachedQueryBudgetStore {
    fn find(&self, code: &str) -> Option<QueryBudget> {
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

    fn save(&self, item: &QueryBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryBudget> {
        self.inner.list()
    }
}

pub fn new_cached_query_budget_store(
    inner: Rc<dyn QueryBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryBudgetStore> {
    Rc::new(CachedQueryBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryBudget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryBudgetQuery {
    pub source: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryBudgetQuery {
    pub fn matches(&self, item: &QueryBudget) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QueryBudget]) -> Vec<QueryBudget> {
        let matched = collections::filter(items, |candidate: &QueryBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_budgets(store: Rc<dyn QueryBudgetStore>, query: &QueryBudgetQuery) -> Vec<QueryBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QueryBudgetStore> = new_memory_query_budget_store();
        let item = new_query_budget("code-979", "title-738");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
