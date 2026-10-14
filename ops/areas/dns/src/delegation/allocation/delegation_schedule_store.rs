use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps delegation schedules by code.
pub trait DelegationScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DelegationSchedule>;
    fn save(&self, item: &DelegationSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DelegationSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelegationScheduleStore {
    pub items: RefCell<BTreeMap<String, DelegationSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl DelegationScheduleStore for MemoryDelegationScheduleStore {
    fn find(&self, code: &str) -> Option<DelegationSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DelegationSchedule) {
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

    fn list(&self) -> Vec<DelegationSchedule> {
        let mut out: Vec<DelegationSchedule> = Vec::new();
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

pub fn new_memory_delegation_schedule_store() -> Rc<dyn DelegationScheduleStore> {
    Rc::new(MemoryDelegationScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDelegationScheduleStore {
    pub inner: Rc<dyn DelegationScheduleStore>,
    pub cache: RefCell<cache::TtlCache<DelegationSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DelegationScheduleStore for CachedDelegationScheduleStore {
    fn find(&self, code: &str) -> Option<DelegationSchedule> {
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

    fn save(&self, item: &DelegationSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DelegationSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_delegation_schedule_store(
    inner: Rc<dyn DelegationScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DelegationScheduleStore> {
    Rc::new(CachedDelegationScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DelegationSchedule>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects delegation schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScheduleQuery {
    pub country: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DelegationScheduleQuery {
    pub fn matches(&self, item: &DelegationSchedule) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DelegationSchedule]) -> Vec<DelegationSchedule> {
        let matched = collections::filter(items, |candidate: &DelegationSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_delegation_schedules(
    store: Rc<dyn DelegationScheduleStore>,
    query: &DelegationScheduleQuery,
) -> Vec<DelegationSchedule> {
    query.apply(&store.list())
}
