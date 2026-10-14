use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log reservations by code.
pub trait FlowLogReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogReservation>;
    fn save(&self, item: &FlowLogReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogReservationStore {
    pub items: RefCell<BTreeMap<String, FlowLogReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogReservationStore for MemoryFlowLogReservationStore {
    fn find(&self, code: &str) -> Option<FlowLogReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogReservation) {
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

    fn list(&self) -> Vec<FlowLogReservation> {
        let mut out: Vec<FlowLogReservation> = Vec::new();
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

pub fn new_memory_flow_log_reservation_store() -> Rc<dyn FlowLogReservationStore> {
    Rc::new(MemoryFlowLogReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogReservationStore {
    pub inner: Rc<dyn FlowLogReservationStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogReservationStore for CachedFlowLogReservationStore {
    fn find(&self, code: &str) -> Option<FlowLogReservation> {
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

    fn save(&self, item: &FlowLogReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogReservation> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_reservation_store(
    inner: Rc<dyn FlowLogReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogReservationStore> {
    Rc::new(CachedFlowLogReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogReservation>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogReservationQuery {
    pub title: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogReservationQuery {
    pub fn matches(&self, item: &FlowLogReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogReservation]) -> Vec<FlowLogReservation> {
        let matched = collections::filter(items, |candidate: &FlowLogReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_reservations(
    store: Rc<dyn FlowLogReservationStore>,
    query: &FlowLogReservationQuery,
) -> Vec<FlowLogReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FlowLogReservationStore> = new_memory_flow_log_reservation_store();
        let item = new_flow_log_reservation("code-862", "title-901");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
