use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal probe schedules by code.
pub trait SeasonalProbeScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalProbeSchedule>;
    fn save(&self, item: &SeasonalProbeSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalProbeSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalProbeScheduleStore {
    pub items: RefCell<BTreeMap<String, SeasonalProbeSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalProbeScheduleStore for MemorySeasonalProbeScheduleStore {
    fn find(&self, code: &str) -> Option<SeasonalProbeSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalProbeSchedule) {
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

    fn list(&self) -> Vec<SeasonalProbeSchedule> {
        let mut out: Vec<SeasonalProbeSchedule> = Vec::new();
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

pub fn new_memory_seasonal_probe_schedule_store() -> Rc<dyn SeasonalProbeScheduleStore> {
    Rc::new(MemorySeasonalProbeScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalProbeScheduleStore {
    pub inner: Rc<dyn SeasonalProbeScheduleStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalProbeSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalProbeScheduleStore for CachedSeasonalProbeScheduleStore {
    fn find(&self, code: &str) -> Option<SeasonalProbeSchedule> {
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

    fn save(&self, item: &SeasonalProbeSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalProbeSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_probe_schedule_store(
    inner: Rc<dyn SeasonalProbeScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalProbeScheduleStore> {
    Rc::new(CachedSeasonalProbeScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalProbeSchedule>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal probe schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeScheduleQuery {
    pub name: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalProbeScheduleQuery {
    pub fn matches(&self, item: &SeasonalProbeSchedule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[SeasonalProbeSchedule]) -> Vec<SeasonalProbeSchedule> {
        let matched = collections::filter(items, |candidate: &SeasonalProbeSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_probe_schedules(
    store: Rc<dyn SeasonalProbeScheduleStore>,
    query: &SeasonalProbeScheduleQuery,
) -> Vec<SeasonalProbeSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalProbeScheduleStore> = new_memory_seasonal_probe_schedule_store();
        let item = new_seasonal_probe_schedule("code-371", "name-119");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
