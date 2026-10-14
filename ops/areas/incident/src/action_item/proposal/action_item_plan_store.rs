use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item plans by code.
pub trait ActionItemPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemPlan>;
    fn save(&self, item: &ActionItemPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemPlanStore {
    pub items: RefCell<BTreeMap<String, ActionItemPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemPlanStore for MemoryActionItemPlanStore {
    fn find(&self, code: &str) -> Option<ActionItemPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemPlan) {
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

    fn list(&self) -> Vec<ActionItemPlan> {
        let mut out: Vec<ActionItemPlan> = Vec::new();
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

pub fn new_memory_action_item_plan_store() -> Rc<dyn ActionItemPlanStore> {
    Rc::new(MemoryActionItemPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemPlanStore {
    pub inner: Rc<dyn ActionItemPlanStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemPlanStore for CachedActionItemPlanStore {
    fn find(&self, code: &str) -> Option<ActionItemPlan> {
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

    fn save(&self, item: &ActionItemPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemPlan> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_plan_store(
    inner: Rc<dyn ActionItemPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemPlanStore> {
    Rc::new(CachedActionItemPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemPlan>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPlanQuery {
    pub name: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemPlanQuery {
    pub fn matches(&self, item: &ActionItemPlan) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ActionItemPlan]) -> Vec<ActionItemPlan> {
        let matched = collections::filter(items, |candidate: &ActionItemPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_plans(store: Rc<dyn ActionItemPlanStore>, query: &ActionItemPlanQuery) -> Vec<ActionItemPlan> {
    query.apply(&store.list())
}
