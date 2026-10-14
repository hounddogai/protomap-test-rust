use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps migration schedules by code.
pub trait MigrationScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MigrationSchedule>;
    fn save(&self, item: &MigrationSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MigrationSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMigrationScheduleStore {
    pub items: RefCell<BTreeMap<String, MigrationSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl MigrationScheduleStore for MemoryMigrationScheduleStore {
    fn find(&self, code: &str) -> Option<MigrationSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MigrationSchedule) {
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

    fn list(&self) -> Vec<MigrationSchedule> {
        let mut out: Vec<MigrationSchedule> = Vec::new();
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

pub fn new_memory_migration_schedule_store() -> Rc<dyn MigrationScheduleStore> {
    Rc::new(MemoryMigrationScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMigrationScheduleStore {
    pub inner: Rc<dyn MigrationScheduleStore>,
    pub cache: RefCell<cache::TtlCache<MigrationSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MigrationScheduleStore for CachedMigrationScheduleStore {
    fn find(&self, code: &str) -> Option<MigrationSchedule> {
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

    fn save(&self, item: &MigrationSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MigrationSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_migration_schedule_store(
    inner: Rc<dyn MigrationScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MigrationScheduleStore> {
    Rc::new(CachedMigrationScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MigrationSchedule>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects migration schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScheduleQuery {
    pub label: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MigrationScheduleQuery {
    pub fn matches(&self, item: &MigrationSchedule) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[MigrationSchedule]) -> Vec<MigrationSchedule> {
        let matched = collections::filter(items, |candidate: &MigrationSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_migration_schedules(
    store: Rc<dyn MigrationScheduleStore>,
    query: &MigrationScheduleQuery,
) -> Vec<MigrationSchedule> {
    query.apply(&store.list())
}
