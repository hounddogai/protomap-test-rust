use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled federation budgets by code.
pub trait ScheduledFederationBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledFederationBudget>;
    fn save(&self, item: &ScheduledFederationBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledFederationBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledFederationBudgetStore {
    pub items: RefCell<BTreeMap<String, ScheduledFederationBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledFederationBudgetStore for MemoryScheduledFederationBudgetStore {
    fn find(&self, code: &str) -> Option<ScheduledFederationBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledFederationBudget) {
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

    fn list(&self) -> Vec<ScheduledFederationBudget> {
        let mut out: Vec<ScheduledFederationBudget> = Vec::new();
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

pub fn new_memory_scheduled_federation_budget_store() -> Rc<dyn ScheduledFederationBudgetStore> {
    Rc::new(MemoryScheduledFederationBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledFederationBudgetStore {
    pub inner: Rc<dyn ScheduledFederationBudgetStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledFederationBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledFederationBudgetStore for CachedScheduledFederationBudgetStore {
    fn find(&self, code: &str) -> Option<ScheduledFederationBudget> {
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

    fn save(&self, item: &ScheduledFederationBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledFederationBudget> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_federation_budget_store(
    inner: Rc<dyn ScheduledFederationBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledFederationBudgetStore> {
    Rc::new(CachedScheduledFederationBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledFederationBudget>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled federation budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationBudgetQuery {
    pub title: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledFederationBudgetQuery {
    pub fn matches(&self, item: &ScheduledFederationBudget) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledFederationBudget]) -> Vec<ScheduledFederationBudget> {
        let matched = collections::filter(items, |candidate: &ScheduledFederationBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_federation_budgets(
    store: Rc<dyn ScheduledFederationBudgetStore>,
    query: &ScheduledFederationBudgetQuery,
) -> Vec<ScheduledFederationBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledFederationBudgetStore> = new_memory_scheduled_federation_budget_store();
        let item = new_scheduled_federation_budget("code-315", "title-623");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
