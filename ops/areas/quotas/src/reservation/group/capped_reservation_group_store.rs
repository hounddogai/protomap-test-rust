use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped reservation groups by code.
pub trait CappedReservationGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedReservationGroup>;
    fn save(&self, item: &CappedReservationGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedReservationGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedReservationGroupStore {
    pub items: RefCell<BTreeMap<String, CappedReservationGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedReservationGroupStore for MemoryCappedReservationGroupStore {
    fn find(&self, code: &str) -> Option<CappedReservationGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedReservationGroup) {
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

    fn list(&self) -> Vec<CappedReservationGroup> {
        let mut out: Vec<CappedReservationGroup> = Vec::new();
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

pub fn new_memory_capped_reservation_group_store() -> Rc<dyn CappedReservationGroupStore> {
    Rc::new(MemoryCappedReservationGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedReservationGroupStore {
    pub inner: Rc<dyn CappedReservationGroupStore>,
    pub cache: RefCell<cache::TtlCache<CappedReservationGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedReservationGroupStore for CachedCappedReservationGroupStore {
    fn find(&self, code: &str) -> Option<CappedReservationGroup> {
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

    fn save(&self, item: &CappedReservationGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedReservationGroup> {
        self.inner.list()
    }
}

pub fn new_cached_capped_reservation_group_store(
    inner: Rc<dyn CappedReservationGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedReservationGroupStore> {
    Rc::new(CachedCappedReservationGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedReservationGroup>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped reservation groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationGroupQuery {
    pub segment: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedReservationGroupQuery {
    pub fn matches(&self, item: &CappedReservationGroup) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedReservationGroup]) -> Vec<CappedReservationGroup> {
        let matched = collections::filter(items, |candidate: &CappedReservationGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_reservation_groups(
    store: Rc<dyn CappedReservationGroupStore>,
    query: &CappedReservationGroupQuery,
) -> Vec<CappedReservationGroup> {
    query.apply(&store.list())
}
