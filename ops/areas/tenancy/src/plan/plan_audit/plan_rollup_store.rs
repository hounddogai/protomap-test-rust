use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan rollups by code.
pub trait PlanRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanRollup>;
    fn save(&self, item: &PlanRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanRollupStore {
    pub items: RefCell<BTreeMap<String, PlanRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanRollupStore for MemoryPlanRollupStore {
    fn find(&self, code: &str) -> Option<PlanRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanRollup) {
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

    fn list(&self) -> Vec<PlanRollup> {
        let mut out: Vec<PlanRollup> = Vec::new();
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

pub fn new_memory_plan_rollup_store() -> Rc<dyn PlanRollupStore> {
    Rc::new(MemoryPlanRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanRollupStore {
    pub inner: Rc<dyn PlanRollupStore>,
    pub cache: RefCell<cache::TtlCache<PlanRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanRollupStore for CachedPlanRollupStore {
    fn find(&self, code: &str) -> Option<PlanRollup> {
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

    fn save(&self, item: &PlanRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanRollup> {
        self.inner.list()
    }
}

pub fn new_cached_plan_rollup_store(
    inner: Rc<dyn PlanRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanRollupStore> {
    Rc::new(CachedPlanRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanRollup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRollupQuery {
    pub category: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanRollupQuery {
    pub fn matches(&self, item: &PlanRollup) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[PlanRollup]) -> Vec<PlanRollup> {
        let matched = collections::filter(items, |candidate: &PlanRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_rollups(store: Rc<dyn PlanRollupStore>, query: &PlanRollupQuery) -> Vec<PlanRollup> {
    query.apply(&store.list())
}
