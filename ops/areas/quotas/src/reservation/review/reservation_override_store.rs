use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps reservation overrides by code.
pub trait ReservationOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReservationOverride>;
    fn save(&self, item: &ReservationOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReservationOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservationOverrideStore {
    pub items: RefCell<BTreeMap<String, ReservationOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl ReservationOverrideStore for MemoryReservationOverrideStore {
    fn find(&self, code: &str) -> Option<ReservationOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReservationOverride) {
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

    fn list(&self) -> Vec<ReservationOverride> {
        let mut out: Vec<ReservationOverride> = Vec::new();
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

pub fn new_memory_reservation_override_store() -> Rc<dyn ReservationOverrideStore> {
    Rc::new(MemoryReservationOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReservationOverrideStore {
    pub inner: Rc<dyn ReservationOverrideStore>,
    pub cache: RefCell<cache::TtlCache<ReservationOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReservationOverrideStore for CachedReservationOverrideStore {
    fn find(&self, code: &str) -> Option<ReservationOverride> {
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

    fn save(&self, item: &ReservationOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReservationOverride> {
        self.inner.list()
    }
}

pub fn new_cached_reservation_override_store(
    inner: Rc<dyn ReservationOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReservationOverrideStore> {
    Rc::new(CachedReservationOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReservationOverride>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects reservation overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationOverrideQuery {
    pub namespace: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReservationOverrideQuery {
    pub fn matches(&self, item: &ReservationOverride) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReservationOverride]) -> Vec<ReservationOverride> {
        let matched = collections::filter(items, |candidate: &ReservationOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_reservation_overrides(
    store: Rc<dyn ReservationOverrideStore>,
    query: &ReservationOverrideQuery,
) -> Vec<ReservationOverride> {
    query.apply(&store.list())
}
