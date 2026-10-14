use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped budget burn cycles by code.
pub trait CappedBudgetBurnCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedBudgetBurnCycle>;
    fn save(&self, item: &CappedBudgetBurnCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedBudgetBurnCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedBudgetBurnCycleStore {
    pub items: RefCell<BTreeMap<String, CappedBudgetBurnCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedBudgetBurnCycleStore for MemoryCappedBudgetBurnCycleStore {
    fn find(&self, code: &str) -> Option<CappedBudgetBurnCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedBudgetBurnCycle) {
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

    fn list(&self) -> Vec<CappedBudgetBurnCycle> {
        let mut out: Vec<CappedBudgetBurnCycle> = Vec::new();
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

pub fn new_memory_capped_budget_burn_cycle_store() -> Rc<dyn CappedBudgetBurnCycleStore> {
    Rc::new(MemoryCappedBudgetBurnCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedBudgetBurnCycleStore {
    pub inner: Rc<dyn CappedBudgetBurnCycleStore>,
    pub cache: RefCell<cache::TtlCache<CappedBudgetBurnCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedBudgetBurnCycleStore for CachedCappedBudgetBurnCycleStore {
    fn find(&self, code: &str) -> Option<CappedBudgetBurnCycle> {
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

    fn save(&self, item: &CappedBudgetBurnCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedBudgetBurnCycle> {
        self.inner.list()
    }
}

pub fn new_cached_capped_budget_burn_cycle_store(
    inner: Rc<dyn CappedBudgetBurnCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedBudgetBurnCycleStore> {
    Rc::new(CachedCappedBudgetBurnCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedBudgetBurnCycle>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped budget burn cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnCycleQuery {
    pub source: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedBudgetBurnCycleQuery {
    pub fn matches(&self, item: &CappedBudgetBurnCycle) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedBudgetBurnCycle]) -> Vec<CappedBudgetBurnCycle> {
        let matched = collections::filter(items, |candidate: &CappedBudgetBurnCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_budget_burn_cycles(
    store: Rc<dyn CappedBudgetBurnCycleStore>,
    query: &CappedBudgetBurnCycleQuery,
) -> Vec<CappedBudgetBurnCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedBudgetBurnCycleStore> = new_memory_capped_budget_burn_cycle_store();
        let item = new_capped_budget_burn_cycle("code-529", "label-826");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
