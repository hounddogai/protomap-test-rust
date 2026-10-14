use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive allowlist budgets by code.
pub trait AdaptiveAllowlistBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveAllowlistBudget>;
    fn save(&self, item: &AdaptiveAllowlistBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveAllowlistBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveAllowlistBudgetStore {
    pub items: RefCell<BTreeMap<String, AdaptiveAllowlistBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveAllowlistBudgetStore for MemoryAdaptiveAllowlistBudgetStore {
    fn find(&self, code: &str) -> Option<AdaptiveAllowlistBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveAllowlistBudget) {
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

    fn list(&self) -> Vec<AdaptiveAllowlistBudget> {
        let mut out: Vec<AdaptiveAllowlistBudget> = Vec::new();
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

pub fn new_memory_adaptive_allowlist_budget_store() -> Rc<dyn AdaptiveAllowlistBudgetStore> {
    Rc::new(MemoryAdaptiveAllowlistBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveAllowlistBudgetStore {
    pub inner: Rc<dyn AdaptiveAllowlistBudgetStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveAllowlistBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveAllowlistBudgetStore for CachedAdaptiveAllowlistBudgetStore {
    fn find(&self, code: &str) -> Option<AdaptiveAllowlistBudget> {
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

    fn save(&self, item: &AdaptiveAllowlistBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveAllowlistBudget> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_allowlist_budget_store(
    inner: Rc<dyn AdaptiveAllowlistBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveAllowlistBudgetStore> {
    Rc::new(CachedAdaptiveAllowlistBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveAllowlistBudget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive allowlist budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBudgetQuery {
    pub channel: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveAllowlistBudgetQuery {
    pub fn matches(&self, item: &AdaptiveAllowlistBudget) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveAllowlistBudget]) -> Vec<AdaptiveAllowlistBudget> {
        let matched = collections::filter(items, |candidate: &AdaptiveAllowlistBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_allowlist_budgets(
    store: Rc<dyn AdaptiveAllowlistBudgetStore>,
    query: &AdaptiveAllowlistBudgetQuery,
) -> Vec<AdaptiveAllowlistBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveAllowlistBudgetStore> = new_memory_adaptive_allowlist_budget_store();
        let item = new_adaptive_allowlist_budget("code-515", "title-50");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
