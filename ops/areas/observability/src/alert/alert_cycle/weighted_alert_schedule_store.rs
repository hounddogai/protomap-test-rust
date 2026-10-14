use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted alert schedules by code.
pub trait WeightedAlertScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedAlertSchedule>;
    fn save(&self, item: &WeightedAlertSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedAlertSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedAlertScheduleStore {
    pub items: RefCell<BTreeMap<String, WeightedAlertSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedAlertScheduleStore for MemoryWeightedAlertScheduleStore {
    fn find(&self, code: &str) -> Option<WeightedAlertSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedAlertSchedule) {
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

    fn list(&self) -> Vec<WeightedAlertSchedule> {
        let mut out: Vec<WeightedAlertSchedule> = Vec::new();
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

pub fn new_memory_weighted_alert_schedule_store() -> Rc<dyn WeightedAlertScheduleStore> {
    Rc::new(MemoryWeightedAlertScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedAlertScheduleStore {
    pub inner: Rc<dyn WeightedAlertScheduleStore>,
    pub cache: RefCell<cache::TtlCache<WeightedAlertSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedAlertScheduleStore for CachedWeightedAlertScheduleStore {
    fn find(&self, code: &str) -> Option<WeightedAlertSchedule> {
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

    fn save(&self, item: &WeightedAlertSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedAlertSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_alert_schedule_store(
    inner: Rc<dyn WeightedAlertScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedAlertScheduleStore> {
    Rc::new(CachedWeightedAlertScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedAlertSchedule>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted alert schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertScheduleQuery {
    pub name: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedAlertScheduleQuery {
    pub fn matches(&self, item: &WeightedAlertSchedule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedAlertSchedule]) -> Vec<WeightedAlertSchedule> {
        let matched = collections::filter(items, |candidate: &WeightedAlertSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_alert_schedules(
    store: Rc<dyn WeightedAlertScheduleStore>,
    query: &WeightedAlertScheduleQuery,
) -> Vec<WeightedAlertSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedAlertScheduleStore> = new_memory_weighted_alert_schedule_store();
        let item = new_weighted_alert_schedule("code-579", "name-92");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
