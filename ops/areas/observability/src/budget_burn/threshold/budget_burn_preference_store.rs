use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps budget burn preferences by code.
pub trait BudgetBurnPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BudgetBurnPreference>;
    fn save(&self, item: &BudgetBurnPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BudgetBurnPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBudgetBurnPreferenceStore {
    pub items: RefCell<BTreeMap<String, BudgetBurnPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl BudgetBurnPreferenceStore for MemoryBudgetBurnPreferenceStore {
    fn find(&self, code: &str) -> Option<BudgetBurnPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BudgetBurnPreference) {
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

    fn list(&self) -> Vec<BudgetBurnPreference> {
        let mut out: Vec<BudgetBurnPreference> = Vec::new();
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

pub fn new_memory_budget_burn_preference_store() -> Rc<dyn BudgetBurnPreferenceStore> {
    Rc::new(MemoryBudgetBurnPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBudgetBurnPreferenceStore {
    pub inner: Rc<dyn BudgetBurnPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<BudgetBurnPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BudgetBurnPreferenceStore for CachedBudgetBurnPreferenceStore {
    fn find(&self, code: &str) -> Option<BudgetBurnPreference> {
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

    fn save(&self, item: &BudgetBurnPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BudgetBurnPreference> {
        self.inner.list()
    }
}

pub fn new_cached_budget_burn_preference_store(
    inner: Rc<dyn BudgetBurnPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BudgetBurnPreferenceStore> {
    Rc::new(CachedBudgetBurnPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BudgetBurnPreference>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects budget burn preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPreferenceQuery {
    pub namespace: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BudgetBurnPreferenceQuery {
    pub fn matches(&self, item: &BudgetBurnPreference) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BudgetBurnPreference]) -> Vec<BudgetBurnPreference> {
        let matched = collections::filter(items, |candidate: &BudgetBurnPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_budget_burn_preferences(
    store: Rc<dyn BudgetBurnPreferenceStore>,
    query: &BudgetBurnPreferenceQuery,
) -> Vec<BudgetBurnPreference> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BudgetBurnPreferenceStore> = new_memory_budget_burn_preference_store();
        let item = new_budget_burn_preference("code-104", "title-405");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
