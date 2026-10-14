use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy rollout budgets by code.
pub trait LegacyRolloutBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyRolloutBudget>;
    fn save(&self, item: &LegacyRolloutBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyRolloutBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyRolloutBudgetStore {
    pub items: RefCell<BTreeMap<String, LegacyRolloutBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyRolloutBudgetStore for MemoryLegacyRolloutBudgetStore {
    fn find(&self, code: &str) -> Option<LegacyRolloutBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyRolloutBudget) {
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

    fn list(&self) -> Vec<LegacyRolloutBudget> {
        let mut out: Vec<LegacyRolloutBudget> = Vec::new();
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

pub fn new_memory_legacy_rollout_budget_store() -> Rc<dyn LegacyRolloutBudgetStore> {
    Rc::new(MemoryLegacyRolloutBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyRolloutBudgetStore {
    pub inner: Rc<dyn LegacyRolloutBudgetStore>,
    pub cache: RefCell<cache::TtlCache<LegacyRolloutBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyRolloutBudgetStore for CachedLegacyRolloutBudgetStore {
    fn find(&self, code: &str) -> Option<LegacyRolloutBudget> {
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

    fn save(&self, item: &LegacyRolloutBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyRolloutBudget> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_rollout_budget_store(
    inner: Rc<dyn LegacyRolloutBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyRolloutBudgetStore> {
    Rc::new(CachedLegacyRolloutBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyRolloutBudget>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy rollout budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutBudgetQuery {
    pub name: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyRolloutBudgetQuery {
    pub fn matches(&self, item: &LegacyRolloutBudget) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[LegacyRolloutBudget]) -> Vec<LegacyRolloutBudget> {
        let matched = collections::filter(items, |candidate: &LegacyRolloutBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_rollout_budgets(
    store: Rc<dyn LegacyRolloutBudgetStore>,
    query: &LegacyRolloutBudgetQuery,
) -> Vec<LegacyRolloutBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyRolloutBudgetStore> = new_memory_legacy_rollout_budget_store();
        let item = new_legacy_rollout_budget("code-45", "name-419");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
