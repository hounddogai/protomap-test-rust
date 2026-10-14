use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy ttl policy schedules by code.
pub trait LegacyTtlPolicyScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyTtlPolicySchedule>;
    fn save(&self, item: &LegacyTtlPolicySchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyTtlPolicySchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyTtlPolicyScheduleStore {
    pub items: RefCell<BTreeMap<String, LegacyTtlPolicySchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyTtlPolicyScheduleStore for MemoryLegacyTtlPolicyScheduleStore {
    fn find(&self, code: &str) -> Option<LegacyTtlPolicySchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyTtlPolicySchedule) {
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

    fn list(&self) -> Vec<LegacyTtlPolicySchedule> {
        let mut out: Vec<LegacyTtlPolicySchedule> = Vec::new();
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

pub fn new_memory_legacy_ttl_policy_schedule_store() -> Rc<dyn LegacyTtlPolicyScheduleStore> {
    Rc::new(MemoryLegacyTtlPolicyScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyTtlPolicyScheduleStore {
    pub inner: Rc<dyn LegacyTtlPolicyScheduleStore>,
    pub cache: RefCell<cache::TtlCache<LegacyTtlPolicySchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyTtlPolicyScheduleStore for CachedLegacyTtlPolicyScheduleStore {
    fn find(&self, code: &str) -> Option<LegacyTtlPolicySchedule> {
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

    fn save(&self, item: &LegacyTtlPolicySchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyTtlPolicySchedule> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_ttl_policy_schedule_store(
    inner: Rc<dyn LegacyTtlPolicyScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyTtlPolicyScheduleStore> {
    Rc::new(CachedLegacyTtlPolicyScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyTtlPolicySchedule>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy ttl policy schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyScheduleQuery {
    pub country: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyTtlPolicyScheduleQuery {
    pub fn matches(&self, item: &LegacyTtlPolicySchedule) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyTtlPolicySchedule]) -> Vec<LegacyTtlPolicySchedule> {
        let matched = collections::filter(items, |candidate: &LegacyTtlPolicySchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_ttl_policy_schedules(
    store: Rc<dyn LegacyTtlPolicyScheduleStore>,
    query: &LegacyTtlPolicyScheduleQuery,
) -> Vec<LegacyTtlPolicySchedule> {
    query.apply(&store.list())
}
