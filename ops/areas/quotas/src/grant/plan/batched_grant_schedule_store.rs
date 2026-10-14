use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched grant schedules by code.
pub trait BatchedGrantScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedGrantSchedule>;
    fn save(&self, item: &BatchedGrantSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedGrantSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedGrantScheduleStore {
    pub items: RefCell<BTreeMap<String, BatchedGrantSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedGrantScheduleStore for MemoryBatchedGrantScheduleStore {
    fn find(&self, code: &str) -> Option<BatchedGrantSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedGrantSchedule) {
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

    fn list(&self) -> Vec<BatchedGrantSchedule> {
        let mut out: Vec<BatchedGrantSchedule> = Vec::new();
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

pub fn new_memory_batched_grant_schedule_store() -> Rc<dyn BatchedGrantScheduleStore> {
    Rc::new(MemoryBatchedGrantScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedGrantScheduleStore {
    pub inner: Rc<dyn BatchedGrantScheduleStore>,
    pub cache: RefCell<cache::TtlCache<BatchedGrantSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedGrantScheduleStore for CachedBatchedGrantScheduleStore {
    fn find(&self, code: &str) -> Option<BatchedGrantSchedule> {
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

    fn save(&self, item: &BatchedGrantSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedGrantSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_batched_grant_schedule_store(
    inner: Rc<dyn BatchedGrantScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedGrantScheduleStore> {
    Rc::new(CachedBatchedGrantScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedGrantSchedule>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched grant schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantScheduleQuery {
    pub locale: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedGrantScheduleQuery {
    pub fn matches(&self, item: &BatchedGrantSchedule) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedGrantSchedule]) -> Vec<BatchedGrantSchedule> {
        let matched = collections::filter(items, |candidate: &BatchedGrantSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_grant_schedules(
    store: Rc<dyn BatchedGrantScheduleStore>,
    query: &BatchedGrantScheduleQuery,
) -> Vec<BatchedGrantSchedule> {
    query.apply(&store.list())
}
