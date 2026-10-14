use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered health check reservations by code.
pub trait TieredHealthCheckReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredHealthCheckReservation>;
    fn save(&self, item: &TieredHealthCheckReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredHealthCheckReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredHealthCheckReservationStore {
    pub items: RefCell<BTreeMap<String, TieredHealthCheckReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredHealthCheckReservationStore for MemoryTieredHealthCheckReservationStore {
    fn find(&self, code: &str) -> Option<TieredHealthCheckReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredHealthCheckReservation) {
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

    fn list(&self) -> Vec<TieredHealthCheckReservation> {
        let mut out: Vec<TieredHealthCheckReservation> = Vec::new();
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

pub fn new_memory_tiered_health_check_reservation_store() -> Rc<dyn TieredHealthCheckReservationStore> {
    Rc::new(MemoryTieredHealthCheckReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredHealthCheckReservationStore {
    pub inner: Rc<dyn TieredHealthCheckReservationStore>,
    pub cache: RefCell<cache::TtlCache<TieredHealthCheckReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredHealthCheckReservationStore for CachedTieredHealthCheckReservationStore {
    fn find(&self, code: &str) -> Option<TieredHealthCheckReservation> {
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

    fn save(&self, item: &TieredHealthCheckReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredHealthCheckReservation> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_health_check_reservation_store(
    inner: Rc<dyn TieredHealthCheckReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredHealthCheckReservationStore> {
    Rc::new(CachedTieredHealthCheckReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredHealthCheckReservation>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered health check reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckReservationQuery {
    pub label: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredHealthCheckReservationQuery {
    pub fn matches(&self, item: &TieredHealthCheckReservation) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredHealthCheckReservation]) -> Vec<TieredHealthCheckReservation> {
        let matched = collections::filter(items, |candidate: &TieredHealthCheckReservation| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_health_check_reservations(
    store: Rc<dyn TieredHealthCheckReservationStore>,
    query: &TieredHealthCheckReservationQuery,
) -> Vec<TieredHealthCheckReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredHealthCheckReservationStore> = new_memory_tiered_health_check_reservation_store();
        let item = new_tiered_health_check_reservation("code-449", "label-146");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
