use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps reservation policies by code.
pub trait ReservationPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReservationPolicy>;
    fn save(&self, item: &ReservationPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReservationPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservationPolicyStore {
    pub items: RefCell<BTreeMap<String, ReservationPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl ReservationPolicyStore for MemoryReservationPolicyStore {
    fn find(&self, code: &str) -> Option<ReservationPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReservationPolicy) {
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

    fn list(&self) -> Vec<ReservationPolicy> {
        let mut out: Vec<ReservationPolicy> = Vec::new();
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

pub fn new_memory_reservation_policy_store() -> Rc<dyn ReservationPolicyStore> {
    Rc::new(MemoryReservationPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReservationPolicyStore {
    pub inner: Rc<dyn ReservationPolicyStore>,
    pub cache: RefCell<cache::TtlCache<ReservationPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReservationPolicyStore for CachedReservationPolicyStore {
    fn find(&self, code: &str) -> Option<ReservationPolicy> {
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

    fn save(&self, item: &ReservationPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReservationPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_reservation_policy_store(
    inner: Rc<dyn ReservationPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReservationPolicyStore> {
    Rc::new(CachedReservationPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReservationPolicy>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects reservation policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPolicyQuery {
    pub category: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReservationPolicyQuery {
    pub fn matches(&self, item: &ReservationPolicy) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReservationPolicy]) -> Vec<ReservationPolicy> {
        let matched = collections::filter(items, |candidate: &ReservationPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_reservation_policies(
    store: Rc<dyn ReservationPolicyStore>,
    query: &ReservationPolicyQuery,
) -> Vec<ReservationPolicy> {
    query.apply(&store.list())
}
