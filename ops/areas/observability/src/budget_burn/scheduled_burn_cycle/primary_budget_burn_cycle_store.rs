use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary budget burn cycles by code.
pub trait PrimaryBudgetBurnCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryBudgetBurnCycle>;
    fn save(&self, item: &PrimaryBudgetBurnCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryBudgetBurnCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryBudgetBurnCycleStore {
    pub items: RefCell<BTreeMap<String, PrimaryBudgetBurnCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryBudgetBurnCycleStore for MemoryPrimaryBudgetBurnCycleStore {
    fn find(&self, code: &str) -> Option<PrimaryBudgetBurnCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryBudgetBurnCycle) {
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

    fn list(&self) -> Vec<PrimaryBudgetBurnCycle> {
        let mut out: Vec<PrimaryBudgetBurnCycle> = Vec::new();
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

pub fn new_memory_primary_budget_burn_cycle_store() -> Rc<dyn PrimaryBudgetBurnCycleStore> {
    Rc::new(MemoryPrimaryBudgetBurnCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryBudgetBurnCycleStore {
    pub inner: Rc<dyn PrimaryBudgetBurnCycleStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryBudgetBurnCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryBudgetBurnCycleStore for CachedPrimaryBudgetBurnCycleStore {
    fn find(&self, code: &str) -> Option<PrimaryBudgetBurnCycle> {
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

    fn save(&self, item: &PrimaryBudgetBurnCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryBudgetBurnCycle> {
        self.inner.list()
    }
}

pub fn new_cached_primary_budget_burn_cycle_store(
    inner: Rc<dyn PrimaryBudgetBurnCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryBudgetBurnCycleStore> {
    Rc::new(CachedPrimaryBudgetBurnCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryBudgetBurnCycle>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary budget burn cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnCycleQuery {
    pub channel: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryBudgetBurnCycleQuery {
    pub fn matches(&self, item: &PrimaryBudgetBurnCycle) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryBudgetBurnCycle]) -> Vec<PrimaryBudgetBurnCycle> {
        let matched = collections::filter(items, |candidate: &PrimaryBudgetBurnCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_budget_burn_cycles(
    store: Rc<dyn PrimaryBudgetBurnCycleStore>,
    query: &PrimaryBudgetBurnCycleQuery,
) -> Vec<PrimaryBudgetBurnCycle> {
    query.apply(&store.list())
}
