use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding budgets by code.
pub trait BindingBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingBudget>;
    fn save(&self, item: &BindingBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingBudgetStore {
    pub items: RefCell<BTreeMap<String, BindingBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingBudgetStore for MemoryBindingBudgetStore {
    fn find(&self, code: &str) -> Option<BindingBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingBudget) {
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

    fn list(&self) -> Vec<BindingBudget> {
        let mut out: Vec<BindingBudget> = Vec::new();
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

pub fn new_memory_binding_budget_store() -> Rc<dyn BindingBudgetStore> {
    Rc::new(MemoryBindingBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingBudgetStore {
    pub inner: Rc<dyn BindingBudgetStore>,
    pub cache: RefCell<cache::TtlCache<BindingBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingBudgetStore for CachedBindingBudgetStore {
    fn find(&self, code: &str) -> Option<BindingBudget> {
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

    fn save(&self, item: &BindingBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingBudget> {
        self.inner.list()
    }
}

pub fn new_cached_binding_budget_store(
    inner: Rc<dyn BindingBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingBudgetStore> {
    Rc::new(CachedBindingBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingBudget>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingBudgetQuery {
    pub scope: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingBudgetQuery {
    pub fn matches(&self, item: &BindingBudget) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingBudget]) -> Vec<BindingBudget> {
        let matched = collections::filter(items, |candidate: &BindingBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_budgets(store: Rc<dyn BindingBudgetStore>, query: &BindingBudgetQuery) -> Vec<BindingBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingBudgetStore> = new_memory_binding_budget_store();
        let item = new_binding_budget("code-499", "title-185");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
