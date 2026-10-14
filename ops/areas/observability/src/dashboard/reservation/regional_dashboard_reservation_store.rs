use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional dashboard reservations by code.
pub trait RegionalDashboardReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalDashboardReservation>;
    fn save(&self, item: &RegionalDashboardReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalDashboardReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalDashboardReservationStore {
    pub items: RefCell<BTreeMap<String, RegionalDashboardReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalDashboardReservationStore for MemoryRegionalDashboardReservationStore {
    fn find(&self, code: &str) -> Option<RegionalDashboardReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalDashboardReservation) {
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

    fn list(&self) -> Vec<RegionalDashboardReservation> {
        let mut out: Vec<RegionalDashboardReservation> = Vec::new();
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

pub fn new_memory_regional_dashboard_reservation_store() -> Rc<dyn RegionalDashboardReservationStore> {
    Rc::new(MemoryRegionalDashboardReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalDashboardReservationStore {
    pub inner: Rc<dyn RegionalDashboardReservationStore>,
    pub cache: RefCell<cache::TtlCache<RegionalDashboardReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalDashboardReservationStore for CachedRegionalDashboardReservationStore {
    fn find(&self, code: &str) -> Option<RegionalDashboardReservation> {
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

    fn save(&self, item: &RegionalDashboardReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalDashboardReservation> {
        self.inner.list()
    }
}

pub fn new_cached_regional_dashboard_reservation_store(
    inner: Rc<dyn RegionalDashboardReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalDashboardReservationStore> {
    Rc::new(CachedRegionalDashboardReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalDashboardReservation>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional dashboard reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservationQuery {
    pub region: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalDashboardReservationQuery {
    pub fn matches(&self, item: &RegionalDashboardReservation) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[RegionalDashboardReservation]) -> Vec<RegionalDashboardReservation> {
        let matched = collections::filter(items, |candidate: &RegionalDashboardReservation| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_dashboard_reservations(
    store: Rc<dyn RegionalDashboardReservationStore>,
    query: &RegionalDashboardReservationQuery,
) -> Vec<RegionalDashboardReservation> {
    query.apply(&store.list())
}
