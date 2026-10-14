use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan overrides by code.
pub trait PlanOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanOverride>;
    fn save(&self, item: &PlanOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanOverrideStore {
    pub items: RefCell<BTreeMap<String, PlanOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanOverrideStore for MemoryPlanOverrideStore {
    fn find(&self, code: &str) -> Option<PlanOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanOverride) {
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

    fn list(&self) -> Vec<PlanOverride> {
        let mut out: Vec<PlanOverride> = Vec::new();
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

pub fn new_memory_plan_override_store() -> Rc<dyn PlanOverrideStore> {
    Rc::new(MemoryPlanOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanOverrideStore {
    pub inner: Rc<dyn PlanOverrideStore>,
    pub cache: RefCell<cache::TtlCache<PlanOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanOverrideStore for CachedPlanOverrideStore {
    fn find(&self, code: &str) -> Option<PlanOverride> {
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

    fn save(&self, item: &PlanOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanOverride> {
        self.inner.list()
    }
}

pub fn new_cached_plan_override_store(
    inner: Rc<dyn PlanOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanOverrideStore> {
    Rc::new(CachedPlanOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanOverride>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanOverrideQuery {
    pub label: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanOverrideQuery {
    pub fn matches(&self, item: &PlanOverride) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PlanOverride]) -> Vec<PlanOverride> {
        let matched = collections::filter(items, |candidate: &PlanOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_overrides(store: Rc<dyn PlanOverrideStore>, query: &PlanOverrideQuery) -> Vec<PlanOverride> {
    query.apply(&store.list())
}
