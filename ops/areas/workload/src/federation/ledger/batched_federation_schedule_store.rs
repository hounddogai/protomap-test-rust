use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched federation schedules by code.
pub trait BatchedFederationScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedFederationSchedule>;
    fn save(&self, item: &BatchedFederationSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedFederationSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedFederationScheduleStore {
    pub items: RefCell<BTreeMap<String, BatchedFederationSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedFederationScheduleStore for MemoryBatchedFederationScheduleStore {
    fn find(&self, code: &str) -> Option<BatchedFederationSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedFederationSchedule) {
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

    fn list(&self) -> Vec<BatchedFederationSchedule> {
        let mut out: Vec<BatchedFederationSchedule> = Vec::new();
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

pub fn new_memory_batched_federation_schedule_store() -> Rc<dyn BatchedFederationScheduleStore> {
    Rc::new(MemoryBatchedFederationScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedFederationScheduleStore {
    pub inner: Rc<dyn BatchedFederationScheduleStore>,
    pub cache: RefCell<cache::TtlCache<BatchedFederationSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedFederationScheduleStore for CachedBatchedFederationScheduleStore {
    fn find(&self, code: &str) -> Option<BatchedFederationSchedule> {
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

    fn save(&self, item: &BatchedFederationSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedFederationSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_batched_federation_schedule_store(
    inner: Rc<dyn BatchedFederationScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedFederationScheduleStore> {
    Rc::new(CachedBatchedFederationScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedFederationSchedule>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched federation schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationScheduleQuery {
    pub name: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedFederationScheduleQuery {
    pub fn matches(&self, item: &BatchedFederationSchedule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[BatchedFederationSchedule]) -> Vec<BatchedFederationSchedule> {
        let matched = collections::filter(items, |candidate: &BatchedFederationSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_federation_schedules(
    store: Rc<dyn BatchedFederationScheduleStore>,
    query: &BatchedFederationScheduleQuery,
) -> Vec<BatchedFederationSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedFederationScheduleStore> = new_memory_batched_federation_schedule_store();
        let item = new_batched_federation_schedule("code-196", "name-844");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
