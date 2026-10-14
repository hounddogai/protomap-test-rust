use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps alert schedules by code.
pub trait AlertScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AlertSchedule>;
    fn save(&self, item: &AlertSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AlertSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAlertScheduleStore {
    pub items: RefCell<BTreeMap<String, AlertSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl AlertScheduleStore for MemoryAlertScheduleStore {
    fn find(&self, code: &str) -> Option<AlertSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AlertSchedule) {
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

    fn list(&self) -> Vec<AlertSchedule> {
        let mut out: Vec<AlertSchedule> = Vec::new();
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

pub fn new_memory_alert_schedule_store() -> Rc<dyn AlertScheduleStore> {
    Rc::new(MemoryAlertScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAlertScheduleStore {
    pub inner: Rc<dyn AlertScheduleStore>,
    pub cache: RefCell<cache::TtlCache<AlertSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AlertScheduleStore for CachedAlertScheduleStore {
    fn find(&self, code: &str) -> Option<AlertSchedule> {
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

    fn save(&self, item: &AlertSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AlertSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_alert_schedule_store(
    inner: Rc<dyn AlertScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AlertScheduleStore> {
    Rc::new(CachedAlertScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AlertSchedule>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects alert schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertScheduleQuery {
    pub label: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AlertScheduleQuery {
    pub fn matches(&self, item: &AlertSchedule) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AlertSchedule]) -> Vec<AlertSchedule> {
        let matched = collections::filter(items, |candidate: &AlertSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_alert_schedules(store: Rc<dyn AlertScheduleStore>, query: &AlertScheduleQuery) -> Vec<AlertSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AlertScheduleStore> = new_memory_alert_schedule_store();
        let item = new_alert_schedule("code-131", "label-123");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
