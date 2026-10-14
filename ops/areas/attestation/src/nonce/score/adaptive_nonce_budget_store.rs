use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive nonce budgets by code.
pub trait AdaptiveNonceBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveNonceBudget>;
    fn save(&self, item: &AdaptiveNonceBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveNonceBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveNonceBudgetStore {
    pub items: RefCell<BTreeMap<String, AdaptiveNonceBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveNonceBudgetStore for MemoryAdaptiveNonceBudgetStore {
    fn find(&self, code: &str) -> Option<AdaptiveNonceBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveNonceBudget) {
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

    fn list(&self) -> Vec<AdaptiveNonceBudget> {
        let mut out: Vec<AdaptiveNonceBudget> = Vec::new();
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

pub fn new_memory_adaptive_nonce_budget_store() -> Rc<dyn AdaptiveNonceBudgetStore> {
    Rc::new(MemoryAdaptiveNonceBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveNonceBudgetStore {
    pub inner: Rc<dyn AdaptiveNonceBudgetStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveNonceBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveNonceBudgetStore for CachedAdaptiveNonceBudgetStore {
    fn find(&self, code: &str) -> Option<AdaptiveNonceBudget> {
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

    fn save(&self, item: &AdaptiveNonceBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveNonceBudget> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_nonce_budget_store(
    inner: Rc<dyn AdaptiveNonceBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveNonceBudgetStore> {
    Rc::new(CachedAdaptiveNonceBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveNonceBudget>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive nonce budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceBudgetQuery {
    pub region: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveNonceBudgetQuery {
    pub fn matches(&self, item: &AdaptiveNonceBudget) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveNonceBudget]) -> Vec<AdaptiveNonceBudget> {
        let matched = collections::filter(items, |candidate: &AdaptiveNonceBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_nonce_budgets(
    store: Rc<dyn AdaptiveNonceBudgetStore>,
    query: &AdaptiveNonceBudgetQuery,
) -> Vec<AdaptiveNonceBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveNonceBudgetStore> = new_memory_adaptive_nonce_budget_store();
        let item = new_adaptive_nonce_budget("code-955", "title-982");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
