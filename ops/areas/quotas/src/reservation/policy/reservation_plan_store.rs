use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps reservation plans by code.
pub trait ReservationPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReservationPlan>;
    fn save(&self, item: &ReservationPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReservationPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservationPlanStore {
    pub items: RefCell<BTreeMap<String, ReservationPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl ReservationPlanStore for MemoryReservationPlanStore {
    fn find(&self, code: &str) -> Option<ReservationPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReservationPlan) {
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

    fn list(&self) -> Vec<ReservationPlan> {
        let mut out: Vec<ReservationPlan> = Vec::new();
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

pub fn new_memory_reservation_plan_store() -> Rc<dyn ReservationPlanStore> {
    Rc::new(MemoryReservationPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReservationPlanStore {
    pub inner: Rc<dyn ReservationPlanStore>,
    pub cache: RefCell<cache::TtlCache<ReservationPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReservationPlanStore for CachedReservationPlanStore {
    fn find(&self, code: &str) -> Option<ReservationPlan> {
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

    fn save(&self, item: &ReservationPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReservationPlan> {
        self.inner.list()
    }
}

pub fn new_cached_reservation_plan_store(
    inner: Rc<dyn ReservationPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReservationPlanStore> {
    Rc::new(CachedReservationPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReservationPlan>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects reservation plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPlanQuery {
    pub source: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReservationPlanQuery {
    pub fn matches(&self, item: &ReservationPlan) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[ReservationPlan]) -> Vec<ReservationPlan> {
        let matched = collections::filter(items, |candidate: &ReservationPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_reservation_plans(
    store: Rc<dyn ReservationPlanStore>,
    query: &ReservationPlanQuery,
) -> Vec<ReservationPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReservationPlanStore> = new_memory_reservation_plan_store();
        let item = new_reservation_plan("code-471", "title-471");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
