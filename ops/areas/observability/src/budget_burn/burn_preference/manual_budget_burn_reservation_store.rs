use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual budget burn reservations by code.
pub trait ManualBudgetBurnReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualBudgetBurnReservation>;
    fn save(&self, item: &ManualBudgetBurnReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualBudgetBurnReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualBudgetBurnReservationStore {
    pub items: RefCell<BTreeMap<String, ManualBudgetBurnReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualBudgetBurnReservationStore for MemoryManualBudgetBurnReservationStore {
    fn find(&self, code: &str) -> Option<ManualBudgetBurnReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualBudgetBurnReservation) {
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

    fn list(&self) -> Vec<ManualBudgetBurnReservation> {
        let mut out: Vec<ManualBudgetBurnReservation> = Vec::new();
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

pub fn new_memory_manual_budget_burn_reservation_store() -> Rc<dyn ManualBudgetBurnReservationStore> {
    Rc::new(MemoryManualBudgetBurnReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualBudgetBurnReservationStore {
    pub inner: Rc<dyn ManualBudgetBurnReservationStore>,
    pub cache: RefCell<cache::TtlCache<ManualBudgetBurnReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualBudgetBurnReservationStore for CachedManualBudgetBurnReservationStore {
    fn find(&self, code: &str) -> Option<ManualBudgetBurnReservation> {
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

    fn save(&self, item: &ManualBudgetBurnReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualBudgetBurnReservation> {
        self.inner.list()
    }
}

pub fn new_cached_manual_budget_burn_reservation_store(
    inner: Rc<dyn ManualBudgetBurnReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualBudgetBurnReservationStore> {
    Rc::new(CachedManualBudgetBurnReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualBudgetBurnReservation>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual budget burn reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnReservationQuery {
    pub scope: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualBudgetBurnReservationQuery {
    pub fn matches(&self, item: &ManualBudgetBurnReservation) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[ManualBudgetBurnReservation]) -> Vec<ManualBudgetBurnReservation> {
        let matched = collections::filter(items, |candidate: &ManualBudgetBurnReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_budget_burn_reservations(
    store: Rc<dyn ManualBudgetBurnReservationStore>,
    query: &ManualBudgetBurnReservationQuery,
) -> Vec<ManualBudgetBurnReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualBudgetBurnReservationStore> = new_memory_manual_budget_burn_reservation_store();
        let item = new_manual_budget_burn_reservation("code-512", "name-376");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
