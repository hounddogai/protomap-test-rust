use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps zone plans by code.
pub trait ZonePlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ZonePlan>;
    fn save(&self, item: &ZonePlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ZonePlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryZonePlanStore {
    pub items: RefCell<BTreeMap<String, ZonePlan>>,
    pub order: RefCell<Vec<String>>,
}

impl ZonePlanStore for MemoryZonePlanStore {
    fn find(&self, code: &str) -> Option<ZonePlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ZonePlan) {
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

    fn list(&self) -> Vec<ZonePlan> {
        let mut out: Vec<ZonePlan> = Vec::new();
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

pub fn new_memory_zone_plan_store() -> Rc<dyn ZonePlanStore> {
    Rc::new(MemoryZonePlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedZonePlanStore {
    pub inner: Rc<dyn ZonePlanStore>,
    pub cache: RefCell<cache::TtlCache<ZonePlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ZonePlanStore for CachedZonePlanStore {
    fn find(&self, code: &str) -> Option<ZonePlan> {
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

    fn save(&self, item: &ZonePlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ZonePlan> {
        self.inner.list()
    }
}

pub fn new_cached_zone_plan_store(
    inner: Rc<dyn ZonePlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ZonePlanStore> {
    Rc::new(CachedZonePlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ZonePlan>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects zone plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ZonePlanQuery {
    pub country: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ZonePlanQuery {
    pub fn matches(&self, item: &ZonePlan) -> bool {
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

    pub fn apply(&self, items: &[ZonePlan]) -> Vec<ZonePlan> {
        let matched = collections::filter(items, |candidate: &ZonePlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_zone_plans(store: Rc<dyn ZonePlanStore>, query: &ZonePlanQuery) -> Vec<ZonePlan> {
    query.apply(&store.list())
}
