use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal budget burn cycles by code.
pub trait SeasonalBudgetBurnCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalBudgetBurnCycle>;
    fn save(&self, item: &SeasonalBudgetBurnCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalBudgetBurnCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalBudgetBurnCycleStore {
    pub items: RefCell<BTreeMap<String, SeasonalBudgetBurnCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalBudgetBurnCycleStore for MemorySeasonalBudgetBurnCycleStore {
    fn find(&self, code: &str) -> Option<SeasonalBudgetBurnCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalBudgetBurnCycle) {
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

    fn list(&self) -> Vec<SeasonalBudgetBurnCycle> {
        let mut out: Vec<SeasonalBudgetBurnCycle> = Vec::new();
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

pub fn new_memory_seasonal_budget_burn_cycle_store() -> Rc<dyn SeasonalBudgetBurnCycleStore> {
    Rc::new(MemorySeasonalBudgetBurnCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalBudgetBurnCycleStore {
    pub inner: Rc<dyn SeasonalBudgetBurnCycleStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalBudgetBurnCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalBudgetBurnCycleStore for CachedSeasonalBudgetBurnCycleStore {
    fn find(&self, code: &str) -> Option<SeasonalBudgetBurnCycle> {
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

    fn save(&self, item: &SeasonalBudgetBurnCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalBudgetBurnCycle> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_budget_burn_cycle_store(
    inner: Rc<dyn SeasonalBudgetBurnCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalBudgetBurnCycleStore> {
    Rc::new(CachedSeasonalBudgetBurnCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalBudgetBurnCycle>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal budget burn cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnCycleQuery {
    pub unit: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalBudgetBurnCycleQuery {
    pub fn matches(&self, item: &SeasonalBudgetBurnCycle) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalBudgetBurnCycle]) -> Vec<SeasonalBudgetBurnCycle> {
        let matched = collections::filter(items, |candidate: &SeasonalBudgetBurnCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_budget_burn_cycles(
    store: Rc<dyn SeasonalBudgetBurnCycleStore>,
    query: &SeasonalBudgetBurnCycleQuery,
) -> Vec<SeasonalBudgetBurnCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalBudgetBurnCycleStore> = new_memory_seasonal_budget_burn_cycle_store();
        let item = new_seasonal_budget_burn_cycle("code-582", "name-468");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
