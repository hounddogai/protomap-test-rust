use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps budget burn variances by code.
pub trait BudgetBurnVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BudgetBurnVariance>;
    fn save(&self, item: &BudgetBurnVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BudgetBurnVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBudgetBurnVarianceStore {
    pub items: RefCell<BTreeMap<String, BudgetBurnVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl BudgetBurnVarianceStore for MemoryBudgetBurnVarianceStore {
    fn find(&self, code: &str) -> Option<BudgetBurnVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BudgetBurnVariance) {
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

    fn list(&self) -> Vec<BudgetBurnVariance> {
        let mut out: Vec<BudgetBurnVariance> = Vec::new();
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

pub fn new_memory_budget_burn_variance_store() -> Rc<dyn BudgetBurnVarianceStore> {
    Rc::new(MemoryBudgetBurnVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBudgetBurnVarianceStore {
    pub inner: Rc<dyn BudgetBurnVarianceStore>,
    pub cache: RefCell<cache::TtlCache<BudgetBurnVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BudgetBurnVarianceStore for CachedBudgetBurnVarianceStore {
    fn find(&self, code: &str) -> Option<BudgetBurnVariance> {
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

    fn save(&self, item: &BudgetBurnVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BudgetBurnVariance> {
        self.inner.list()
    }
}

pub fn new_cached_budget_burn_variance_store(
    inner: Rc<dyn BudgetBurnVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BudgetBurnVarianceStore> {
    Rc::new(CachedBudgetBurnVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BudgetBurnVariance>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects budget burn variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnVarianceQuery {
    pub unit: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BudgetBurnVarianceQuery {
    pub fn matches(&self, item: &BudgetBurnVariance) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BudgetBurnVariance]) -> Vec<BudgetBurnVariance> {
        let matched = collections::filter(items, |candidate: &BudgetBurnVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_budget_burn_variances(
    store: Rc<dyn BudgetBurnVarianceStore>,
    query: &BudgetBurnVarianceQuery,
) -> Vec<BudgetBurnVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BudgetBurnVarianceStore> = new_memory_budget_burn_variance_store();
        let item = new_budget_burn_variance("code-694", "label-189");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
