use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy page plans by code.
pub trait LegacyPagePlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyPagePlan>;
    fn save(&self, item: &LegacyPagePlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyPagePlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyPagePlanStore {
    pub items: RefCell<BTreeMap<String, LegacyPagePlan>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyPagePlanStore for MemoryLegacyPagePlanStore {
    fn find(&self, code: &str) -> Option<LegacyPagePlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyPagePlan) {
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

    fn list(&self) -> Vec<LegacyPagePlan> {
        let mut out: Vec<LegacyPagePlan> = Vec::new();
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

pub fn new_memory_legacy_page_plan_store() -> Rc<dyn LegacyPagePlanStore> {
    Rc::new(MemoryLegacyPagePlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyPagePlanStore {
    pub inner: Rc<dyn LegacyPagePlanStore>,
    pub cache: RefCell<cache::TtlCache<LegacyPagePlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyPagePlanStore for CachedLegacyPagePlanStore {
    fn find(&self, code: &str) -> Option<LegacyPagePlan> {
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

    fn save(&self, item: &LegacyPagePlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyPagePlan> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_page_plan_store(
    inner: Rc<dyn LegacyPagePlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyPagePlanStore> {
    Rc::new(CachedLegacyPagePlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyPagePlan>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy page plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPagePlanQuery {
    pub name: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyPagePlanQuery {
    pub fn matches(&self, item: &LegacyPagePlan) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyPagePlan]) -> Vec<LegacyPagePlan> {
        let matched = collections::filter(items, |candidate: &LegacyPagePlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_page_plans(store: Rc<dyn LegacyPagePlanStore>, query: &LegacyPagePlanQuery) -> Vec<LegacyPagePlan> {
    query.apply(&store.list())
}
