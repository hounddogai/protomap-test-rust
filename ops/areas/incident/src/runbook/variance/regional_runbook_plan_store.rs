use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional runbook plans by code.
pub trait RegionalRunbookPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalRunbookPlan>;
    fn save(&self, item: &RegionalRunbookPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalRunbookPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalRunbookPlanStore {
    pub items: RefCell<BTreeMap<String, RegionalRunbookPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalRunbookPlanStore for MemoryRegionalRunbookPlanStore {
    fn find(&self, code: &str) -> Option<RegionalRunbookPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalRunbookPlan) {
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

    fn list(&self) -> Vec<RegionalRunbookPlan> {
        let mut out: Vec<RegionalRunbookPlan> = Vec::new();
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

pub fn new_memory_regional_runbook_plan_store() -> Rc<dyn RegionalRunbookPlanStore> {
    Rc::new(MemoryRegionalRunbookPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalRunbookPlanStore {
    pub inner: Rc<dyn RegionalRunbookPlanStore>,
    pub cache: RefCell<cache::TtlCache<RegionalRunbookPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalRunbookPlanStore for CachedRegionalRunbookPlanStore {
    fn find(&self, code: &str) -> Option<RegionalRunbookPlan> {
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

    fn save(&self, item: &RegionalRunbookPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalRunbookPlan> {
        self.inner.list()
    }
}

pub fn new_cached_regional_runbook_plan_store(
    inner: Rc<dyn RegionalRunbookPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalRunbookPlanStore> {
    Rc::new(CachedRegionalRunbookPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalRunbookPlan>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional runbook plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookPlanQuery {
    pub label: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalRunbookPlanQuery {
    pub fn matches(&self, item: &RegionalRunbookPlan) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalRunbookPlan]) -> Vec<RegionalRunbookPlan> {
        let matched = collections::filter(items, |candidate: &RegionalRunbookPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_runbook_plans(
    store: Rc<dyn RegionalRunbookPlanStore>,
    query: &RegionalRunbookPlanQuery,
) -> Vec<RegionalRunbookPlan> {
    query.apply(&store.list())
}
