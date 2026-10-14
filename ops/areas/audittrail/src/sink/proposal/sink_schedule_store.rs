use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink schedules by code.
pub trait SinkScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkSchedule>;
    fn save(&self, item: &SinkSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkScheduleStore {
    pub items: RefCell<BTreeMap<String, SinkSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkScheduleStore for MemorySinkScheduleStore {
    fn find(&self, code: &str) -> Option<SinkSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkSchedule) {
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

    fn list(&self) -> Vec<SinkSchedule> {
        let mut out: Vec<SinkSchedule> = Vec::new();
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

pub fn new_memory_sink_schedule_store() -> Rc<dyn SinkScheduleStore> {
    Rc::new(MemorySinkScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkScheduleStore {
    pub inner: Rc<dyn SinkScheduleStore>,
    pub cache: RefCell<cache::TtlCache<SinkSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkScheduleStore for CachedSinkScheduleStore {
    fn find(&self, code: &str) -> Option<SinkSchedule> {
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

    fn save(&self, item: &SinkSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_sink_schedule_store(
    inner: Rc<dyn SinkScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkScheduleStore> {
    Rc::new(CachedSinkScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkSchedule>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkScheduleQuery {
    pub segment: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkScheduleQuery {
    pub fn matches(&self, item: &SinkSchedule) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SinkSchedule]) -> Vec<SinkSchedule> {
        let matched = collections::filter(items, |candidate: &SinkSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_schedules(store: Rc<dyn SinkScheduleStore>, query: &SinkScheduleQuery) -> Vec<SinkSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SinkScheduleStore> = new_memory_sink_schedule_store();
        let item = new_sink_schedule("code-34", "title-500");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
