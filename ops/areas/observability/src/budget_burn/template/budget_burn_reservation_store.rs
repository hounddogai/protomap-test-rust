use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps budget burn reservations by code.
pub trait BudgetBurnReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BudgetBurnReservation>;
    fn save(&self, item: &BudgetBurnReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BudgetBurnReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBudgetBurnReservationStore {
    pub items: RefCell<BTreeMap<String, BudgetBurnReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl BudgetBurnReservationStore for MemoryBudgetBurnReservationStore {
    fn find(&self, code: &str) -> Option<BudgetBurnReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BudgetBurnReservation) {
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

    fn list(&self) -> Vec<BudgetBurnReservation> {
        let mut out: Vec<BudgetBurnReservation> = Vec::new();
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

pub fn new_memory_budget_burn_reservation_store() -> Rc<dyn BudgetBurnReservationStore> {
    Rc::new(MemoryBudgetBurnReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBudgetBurnReservationStore {
    pub inner: Rc<dyn BudgetBurnReservationStore>,
    pub cache: RefCell<cache::TtlCache<BudgetBurnReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BudgetBurnReservationStore for CachedBudgetBurnReservationStore {
    fn find(&self, code: &str) -> Option<BudgetBurnReservation> {
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

    fn save(&self, item: &BudgetBurnReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BudgetBurnReservation> {
        self.inner.list()
    }
}

pub fn new_cached_budget_burn_reservation_store(
    inner: Rc<dyn BudgetBurnReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BudgetBurnReservationStore> {
    Rc::new(CachedBudgetBurnReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BudgetBurnReservation>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects budget burn reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReservationQuery {
    pub name: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BudgetBurnReservationQuery {
    pub fn matches(&self, item: &BudgetBurnReservation) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BudgetBurnReservation]) -> Vec<BudgetBurnReservation> {
        let matched = collections::filter(items, |candidate: &BudgetBurnReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_budget_burn_reservations(
    store: Rc<dyn BudgetBurnReservationStore>,
    query: &BudgetBurnReservationQuery,
) -> Vec<BudgetBurnReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BudgetBurnReservationStore> = new_memory_budget_burn_reservation_store();
        let item = new_budget_burn_reservation("code-739", "name-571");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
