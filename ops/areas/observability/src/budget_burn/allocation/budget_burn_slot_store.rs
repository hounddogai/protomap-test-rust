use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps budget burn slots by code.
pub trait BudgetBurnSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BudgetBurnSlot>;
    fn save(&self, item: &BudgetBurnSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BudgetBurnSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBudgetBurnSlotStore {
    pub items: RefCell<BTreeMap<String, BudgetBurnSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl BudgetBurnSlotStore for MemoryBudgetBurnSlotStore {
    fn find(&self, code: &str) -> Option<BudgetBurnSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BudgetBurnSlot) {
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

    fn list(&self) -> Vec<BudgetBurnSlot> {
        let mut out: Vec<BudgetBurnSlot> = Vec::new();
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

pub fn new_memory_budget_burn_slot_store() -> Rc<dyn BudgetBurnSlotStore> {
    Rc::new(MemoryBudgetBurnSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBudgetBurnSlotStore {
    pub inner: Rc<dyn BudgetBurnSlotStore>,
    pub cache: RefCell<cache::TtlCache<BudgetBurnSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BudgetBurnSlotStore for CachedBudgetBurnSlotStore {
    fn find(&self, code: &str) -> Option<BudgetBurnSlot> {
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

    fn save(&self, item: &BudgetBurnSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BudgetBurnSlot> {
        self.inner.list()
    }
}

pub fn new_cached_budget_burn_slot_store(
    inner: Rc<dyn BudgetBurnSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BudgetBurnSlotStore> {
    Rc::new(CachedBudgetBurnSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BudgetBurnSlot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects budget burn slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSlotQuery {
    pub country: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BudgetBurnSlotQuery {
    pub fn matches(&self, item: &BudgetBurnSlot) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BudgetBurnSlot]) -> Vec<BudgetBurnSlot> {
        let matched = collections::filter(items, |candidate: &BudgetBurnSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_budget_burn_slots(store: Rc<dyn BudgetBurnSlotStore>, query: &BudgetBurnSlotQuery) -> Vec<BudgetBurnSlot> {
    query.apply(&store.list())
}
