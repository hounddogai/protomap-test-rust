use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity schedules by code.
pub trait IdentityScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentitySchedule>;
    fn save(&self, item: &IdentitySchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentitySchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityScheduleStore {
    pub items: RefCell<BTreeMap<String, IdentitySchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityScheduleStore for MemoryIdentityScheduleStore {
    fn find(&self, code: &str) -> Option<IdentitySchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentitySchedule) {
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

    fn list(&self) -> Vec<IdentitySchedule> {
        let mut out: Vec<IdentitySchedule> = Vec::new();
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

pub fn new_memory_identity_schedule_store() -> Rc<dyn IdentityScheduleStore> {
    Rc::new(MemoryIdentityScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityScheduleStore {
    pub inner: Rc<dyn IdentityScheduleStore>,
    pub cache: RefCell<cache::TtlCache<IdentitySchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityScheduleStore for CachedIdentityScheduleStore {
    fn find(&self, code: &str) -> Option<IdentitySchedule> {
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

    fn save(&self, item: &IdentitySchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentitySchedule> {
        self.inner.list()
    }
}

pub fn new_cached_identity_schedule_store(
    inner: Rc<dyn IdentityScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityScheduleStore> {
    Rc::new(CachedIdentityScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentitySchedule>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScheduleQuery {
    pub country: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityScheduleQuery {
    pub fn matches(&self, item: &IdentitySchedule) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IdentitySchedule]) -> Vec<IdentitySchedule> {
        let matched = collections::filter(items, |candidate: &IdentitySchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_schedules(
    store: Rc<dyn IdentityScheduleStore>,
    query: &IdentityScheduleQuery,
) -> Vec<IdentitySchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IdentityScheduleStore> = new_memory_identity_schedule_store();
        let item = new_identity_schedule("code-752", "name-672");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
