use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps limit schedules by code.
pub trait LimitScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LimitSchedule>;
    fn save(&self, item: &LimitSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LimitSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLimitScheduleStore {
    pub items: RefCell<BTreeMap<String, LimitSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl LimitScheduleStore for MemoryLimitScheduleStore {
    fn find(&self, code: &str) -> Option<LimitSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LimitSchedule) {
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

    fn list(&self) -> Vec<LimitSchedule> {
        let mut out: Vec<LimitSchedule> = Vec::new();
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

pub fn new_memory_limit_schedule_store() -> Rc<dyn LimitScheduleStore> {
    Rc::new(MemoryLimitScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLimitScheduleStore {
    pub inner: Rc<dyn LimitScheduleStore>,
    pub cache: RefCell<cache::TtlCache<LimitSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LimitScheduleStore for CachedLimitScheduleStore {
    fn find(&self, code: &str) -> Option<LimitSchedule> {
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

    fn save(&self, item: &LimitSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LimitSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_limit_schedule_store(
    inner: Rc<dyn LimitScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LimitScheduleStore> {
    Rc::new(CachedLimitScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LimitSchedule>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects limit schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitScheduleQuery {
    pub label: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LimitScheduleQuery {
    pub fn matches(&self, item: &LimitSchedule) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LimitSchedule]) -> Vec<LimitSchedule> {
        let matched = collections::filter(items, |candidate: &LimitSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_limit_schedules(store: Rc<dyn LimitScheduleStore>, query: &LimitScheduleQuery) -> Vec<LimitSchedule> {
    query.apply(&store.list())
}
