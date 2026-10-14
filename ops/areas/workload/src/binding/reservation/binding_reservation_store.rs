use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding reservations by code.
pub trait BindingReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingReservation>;
    fn save(&self, item: &BindingReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingReservationStore {
    pub items: RefCell<BTreeMap<String, BindingReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingReservationStore for MemoryBindingReservationStore {
    fn find(&self, code: &str) -> Option<BindingReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingReservation) {
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

    fn list(&self) -> Vec<BindingReservation> {
        let mut out: Vec<BindingReservation> = Vec::new();
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

pub fn new_memory_binding_reservation_store() -> Rc<dyn BindingReservationStore> {
    Rc::new(MemoryBindingReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingReservationStore {
    pub inner: Rc<dyn BindingReservationStore>,
    pub cache: RefCell<cache::TtlCache<BindingReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingReservationStore for CachedBindingReservationStore {
    fn find(&self, code: &str) -> Option<BindingReservation> {
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

    fn save(&self, item: &BindingReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingReservation> {
        self.inner.list()
    }
}

pub fn new_cached_binding_reservation_store(
    inner: Rc<dyn BindingReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingReservationStore> {
    Rc::new(CachedBindingReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingReservation>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingReservationQuery {
    pub channel: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingReservationQuery {
    pub fn matches(&self, item: &BindingReservation) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingReservation]) -> Vec<BindingReservation> {
        let matched = collections::filter(items, |candidate: &BindingReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_reservations(
    store: Rc<dyn BindingReservationStore>,
    query: &BindingReservationQuery,
) -> Vec<BindingReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingReservationStore> = new_memory_binding_reservation_store();
        let item = new_binding_reservation("code-682", "label-247");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
