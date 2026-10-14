use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query schedules by code.
pub trait QueryScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QuerySchedule>;
    fn save(&self, item: &QuerySchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QuerySchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryScheduleStore {
    pub items: RefCell<BTreeMap<String, QuerySchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryScheduleStore for MemoryQueryScheduleStore {
    fn find(&self, code: &str) -> Option<QuerySchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QuerySchedule) {
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

    fn list(&self) -> Vec<QuerySchedule> {
        let mut out: Vec<QuerySchedule> = Vec::new();
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

pub fn new_memory_query_schedule_store() -> Rc<dyn QueryScheduleStore> {
    Rc::new(MemoryQueryScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryScheduleStore {
    pub inner: Rc<dyn QueryScheduleStore>,
    pub cache: RefCell<cache::TtlCache<QuerySchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryScheduleStore for CachedQueryScheduleStore {
    fn find(&self, code: &str) -> Option<QuerySchedule> {
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

    fn save(&self, item: &QuerySchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QuerySchedule> {
        self.inner.list()
    }
}

pub fn new_cached_query_schedule_store(
    inner: Rc<dyn QueryScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryScheduleStore> {
    Rc::new(CachedQueryScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QuerySchedule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryScheduleQuery {
    pub label: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryScheduleQuery {
    pub fn matches(&self, item: &QuerySchedule) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QuerySchedule]) -> Vec<QuerySchedule> {
        let matched = collections::filter(items, |candidate: &QuerySchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_schedules(store: Rc<dyn QueryScheduleStore>, query: &QueryScheduleQuery) -> Vec<QuerySchedule> {
    query.apply(&store.list())
}
