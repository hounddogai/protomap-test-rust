use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending plan templates by code.
pub trait PendingPlanTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingPlanTemplate>;
    fn save(&self, item: &PendingPlanTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingPlanTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingPlanTemplateStore {
    pub items: RefCell<BTreeMap<String, PendingPlanTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingPlanTemplateStore for MemoryPendingPlanTemplateStore {
    fn find(&self, code: &str) -> Option<PendingPlanTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingPlanTemplate) {
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

    fn list(&self) -> Vec<PendingPlanTemplate> {
        let mut out: Vec<PendingPlanTemplate> = Vec::new();
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

pub fn new_memory_pending_plan_template_store() -> Rc<dyn PendingPlanTemplateStore> {
    Rc::new(MemoryPendingPlanTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingPlanTemplateStore {
    pub inner: Rc<dyn PendingPlanTemplateStore>,
    pub cache: RefCell<cache::TtlCache<PendingPlanTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingPlanTemplateStore for CachedPendingPlanTemplateStore {
    fn find(&self, code: &str) -> Option<PendingPlanTemplate> {
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

    fn save(&self, item: &PendingPlanTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingPlanTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_pending_plan_template_store(
    inner: Rc<dyn PendingPlanTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingPlanTemplateStore> {
    Rc::new(CachedPendingPlanTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingPlanTemplate>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending plan templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanTemplateQuery {
    pub name: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingPlanTemplateQuery {
    pub fn matches(&self, item: &PendingPlanTemplate) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingPlanTemplate]) -> Vec<PendingPlanTemplate> {
        let matched = collections::filter(items, |candidate: &PendingPlanTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_plan_templates(
    store: Rc<dyn PendingPlanTemplateStore>,
    query: &PendingPlanTemplateQuery,
) -> Vec<PendingPlanTemplate> {
    query.apply(&store.list())
}
