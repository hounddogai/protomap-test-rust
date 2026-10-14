use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default reservation schedules by code.
pub trait DefaultReservationScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultReservationSchedule>;
    fn save(&self, item: &DefaultReservationSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultReservationSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultReservationScheduleStore {
    pub items: RefCell<BTreeMap<String, DefaultReservationSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultReservationScheduleStore for MemoryDefaultReservationScheduleStore {
    fn find(&self, code: &str) -> Option<DefaultReservationSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultReservationSchedule) {
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

    fn list(&self) -> Vec<DefaultReservationSchedule> {
        let mut out: Vec<DefaultReservationSchedule> = Vec::new();
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

pub fn new_memory_default_reservation_schedule_store() -> Rc<dyn DefaultReservationScheduleStore> {
    Rc::new(MemoryDefaultReservationScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultReservationScheduleStore {
    pub inner: Rc<dyn DefaultReservationScheduleStore>,
    pub cache: RefCell<cache::TtlCache<DefaultReservationSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultReservationScheduleStore for CachedDefaultReservationScheduleStore {
    fn find(&self, code: &str) -> Option<DefaultReservationSchedule> {
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

    fn save(&self, item: &DefaultReservationSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultReservationSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_default_reservation_schedule_store(
    inner: Rc<dyn DefaultReservationScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultReservationScheduleStore> {
    Rc::new(CachedDefaultReservationScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultReservationSchedule>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default reservation schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationScheduleQuery {
    pub unit: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultReservationScheduleQuery {
    pub fn matches(&self, item: &DefaultReservationSchedule) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultReservationSchedule]) -> Vec<DefaultReservationSchedule> {
        let matched = collections::filter(items, |candidate: &DefaultReservationSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_reservation_schedules(
    store: Rc<dyn DefaultReservationScheduleStore>,
    query: &DefaultReservationScheduleQuery,
) -> Vec<DefaultReservationSchedule> {
    query.apply(&store.list())
}
