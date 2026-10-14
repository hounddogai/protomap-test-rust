use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling federation schedules by code.
pub trait RollingFederationScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingFederationSchedule>;
    fn save(&self, item: &RollingFederationSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingFederationSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingFederationScheduleStore {
    pub items: RefCell<BTreeMap<String, RollingFederationSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingFederationScheduleStore for MemoryRollingFederationScheduleStore {
    fn find(&self, code: &str) -> Option<RollingFederationSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingFederationSchedule) {
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

    fn list(&self) -> Vec<RollingFederationSchedule> {
        let mut out: Vec<RollingFederationSchedule> = Vec::new();
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

pub fn new_memory_rolling_federation_schedule_store() -> Rc<dyn RollingFederationScheduleStore> {
    Rc::new(MemoryRollingFederationScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingFederationScheduleStore {
    pub inner: Rc<dyn RollingFederationScheduleStore>,
    pub cache: RefCell<cache::TtlCache<RollingFederationSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingFederationScheduleStore for CachedRollingFederationScheduleStore {
    fn find(&self, code: &str) -> Option<RollingFederationSchedule> {
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

    fn save(&self, item: &RollingFederationSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingFederationSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_federation_schedule_store(
    inner: Rc<dyn RollingFederationScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingFederationScheduleStore> {
    Rc::new(CachedRollingFederationScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingFederationSchedule>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling federation schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationScheduleQuery {
    pub name: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingFederationScheduleQuery {
    pub fn matches(&self, item: &RollingFederationSchedule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingFederationSchedule]) -> Vec<RollingFederationSchedule> {
        let matched = collections::filter(items, |candidate: &RollingFederationSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_federation_schedules(
    store: Rc<dyn RollingFederationScheduleStore>,
    query: &RollingFederationScheduleQuery,
) -> Vec<RollingFederationSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingFederationScheduleStore> = new_memory_rolling_federation_schedule_store();
        let item = new_rolling_federation_schedule("code-858", "name-457");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
