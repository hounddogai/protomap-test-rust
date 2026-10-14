use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched budget burn rollups by code.
pub trait BatchedBudgetBurnRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedBudgetBurnRollup>;
    fn save(&self, item: &BatchedBudgetBurnRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedBudgetBurnRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedBudgetBurnRollupStore {
    pub items: RefCell<BTreeMap<String, BatchedBudgetBurnRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedBudgetBurnRollupStore for MemoryBatchedBudgetBurnRollupStore {
    fn find(&self, code: &str) -> Option<BatchedBudgetBurnRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedBudgetBurnRollup) {
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

    fn list(&self) -> Vec<BatchedBudgetBurnRollup> {
        let mut out: Vec<BatchedBudgetBurnRollup> = Vec::new();
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

pub fn new_memory_batched_budget_burn_rollup_store() -> Rc<dyn BatchedBudgetBurnRollupStore> {
    Rc::new(MemoryBatchedBudgetBurnRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedBudgetBurnRollupStore {
    pub inner: Rc<dyn BatchedBudgetBurnRollupStore>,
    pub cache: RefCell<cache::TtlCache<BatchedBudgetBurnRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedBudgetBurnRollupStore for CachedBatchedBudgetBurnRollupStore {
    fn find(&self, code: &str) -> Option<BatchedBudgetBurnRollup> {
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

    fn save(&self, item: &BatchedBudgetBurnRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedBudgetBurnRollup> {
        self.inner.list()
    }
}

pub fn new_cached_batched_budget_burn_rollup_store(
    inner: Rc<dyn BatchedBudgetBurnRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedBudgetBurnRollupStore> {
    Rc::new(CachedBatchedBudgetBurnRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedBudgetBurnRollup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched budget burn rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnRollupQuery {
    pub namespace: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedBudgetBurnRollupQuery {
    pub fn matches(&self, item: &BatchedBudgetBurnRollup) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedBudgetBurnRollup]) -> Vec<BatchedBudgetBurnRollup> {
        let matched = collections::filter(items, |candidate: &BatchedBudgetBurnRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_budget_burn_rollups(
    store: Rc<dyn BatchedBudgetBurnRollupStore>,
    query: &BatchedBudgetBurnRollupQuery,
) -> Vec<BatchedBudgetBurnRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedBudgetBurnRollupStore> = new_memory_batched_budget_burn_rollup_store();
        let item = new_batched_budget_burn_rollup("code-38", "name-711");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
