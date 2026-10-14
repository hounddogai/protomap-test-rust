use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled slo requests by code.
pub trait ScheduledSloRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledSloRequest>;
    fn save(&self, item: &ScheduledSloRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledSloRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledSloRequestStore {
    pub items: RefCell<BTreeMap<String, ScheduledSloRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledSloRequestStore for MemoryScheduledSloRequestStore {
    fn find(&self, code: &str) -> Option<ScheduledSloRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledSloRequest) {
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

    fn list(&self) -> Vec<ScheduledSloRequest> {
        let mut out: Vec<ScheduledSloRequest> = Vec::new();
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

pub fn new_memory_scheduled_slo_request_store() -> Rc<dyn ScheduledSloRequestStore> {
    Rc::new(MemoryScheduledSloRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledSloRequestStore {
    pub inner: Rc<dyn ScheduledSloRequestStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledSloRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledSloRequestStore for CachedScheduledSloRequestStore {
    fn find(&self, code: &str) -> Option<ScheduledSloRequest> {
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

    fn save(&self, item: &ScheduledSloRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledSloRequest> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_slo_request_store(
    inner: Rc<dyn ScheduledSloRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledSloRequestStore> {
    Rc::new(CachedScheduledSloRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledSloRequest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled slo requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRequestQuery {
    pub unit: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledSloRequestQuery {
    pub fn matches(&self, item: &ScheduledSloRequest) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledSloRequest]) -> Vec<ScheduledSloRequest> {
        let matched = collections::filter(items, |candidate: &ScheduledSloRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_slo_requests(
    store: Rc<dyn ScheduledSloRequestStore>,
    query: &ScheduledSloRequestQuery,
) -> Vec<ScheduledSloRequest> {
    query.apply(&store.list())
}
