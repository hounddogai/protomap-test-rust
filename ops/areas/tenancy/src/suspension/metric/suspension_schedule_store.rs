use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension schedules by code.
pub trait SuspensionScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionSchedule>;
    fn save(&self, item: &SuspensionSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionScheduleStore {
    pub items: RefCell<BTreeMap<String, SuspensionSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionScheduleStore for MemorySuspensionScheduleStore {
    fn find(&self, code: &str) -> Option<SuspensionSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionSchedule) {
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

    fn list(&self) -> Vec<SuspensionSchedule> {
        let mut out: Vec<SuspensionSchedule> = Vec::new();
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

pub fn new_memory_suspension_schedule_store() -> Rc<dyn SuspensionScheduleStore> {
    Rc::new(MemorySuspensionScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionScheduleStore {
    pub inner: Rc<dyn SuspensionScheduleStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionScheduleStore for CachedSuspensionScheduleStore {
    fn find(&self, code: &str) -> Option<SuspensionSchedule> {
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

    fn save(&self, item: &SuspensionSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_schedule_store(
    inner: Rc<dyn SuspensionScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionScheduleStore> {
    Rc::new(CachedSuspensionScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionSchedule>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScheduleQuery {
    pub country: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionScheduleQuery {
    pub fn matches(&self, item: &SuspensionSchedule) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SuspensionSchedule]) -> Vec<SuspensionSchedule> {
        let matched = collections::filter(items, |candidate: &SuspensionSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_schedules(
    store: Rc<dyn SuspensionScheduleStore>,
    query: &SuspensionScheduleQuery,
) -> Vec<SuspensionSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SuspensionScheduleStore> = new_memory_suspension_schedule_store();
        let item = new_suspension_schedule("code-816", "title-347");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
