use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient rule reservations by code.
pub trait LenientRuleReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientRuleReservation>;
    fn save(&self, item: &LenientRuleReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientRuleReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientRuleReservationStore {
    pub items: RefCell<BTreeMap<String, LenientRuleReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientRuleReservationStore for MemoryLenientRuleReservationStore {
    fn find(&self, code: &str) -> Option<LenientRuleReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientRuleReservation) {
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

    fn list(&self) -> Vec<LenientRuleReservation> {
        let mut out: Vec<LenientRuleReservation> = Vec::new();
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

pub fn new_memory_lenient_rule_reservation_store() -> Rc<dyn LenientRuleReservationStore> {
    Rc::new(MemoryLenientRuleReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientRuleReservationStore {
    pub inner: Rc<dyn LenientRuleReservationStore>,
    pub cache: RefCell<cache::TtlCache<LenientRuleReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientRuleReservationStore for CachedLenientRuleReservationStore {
    fn find(&self, code: &str) -> Option<LenientRuleReservation> {
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

    fn save(&self, item: &LenientRuleReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientRuleReservation> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_rule_reservation_store(
    inner: Rc<dyn LenientRuleReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientRuleReservationStore> {
    Rc::new(CachedLenientRuleReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientRuleReservation>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient rule reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleReservationQuery {
    pub title: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientRuleReservationQuery {
    pub fn matches(&self, item: &LenientRuleReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientRuleReservation]) -> Vec<LenientRuleReservation> {
        let matched = collections::filter(items, |candidate: &LenientRuleReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_rule_reservations(
    store: Rc<dyn LenientRuleReservationStore>,
    query: &LenientRuleReservationQuery,
) -> Vec<LenientRuleReservation> {
    query.apply(&store.list())
}
