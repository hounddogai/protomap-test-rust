use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive flow log templates by code.
pub trait AdaptiveFlowLogTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveFlowLogTemplate>;
    fn save(&self, item: &AdaptiveFlowLogTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveFlowLogTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveFlowLogTemplateStore {
    pub items: RefCell<BTreeMap<String, AdaptiveFlowLogTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveFlowLogTemplateStore for MemoryAdaptiveFlowLogTemplateStore {
    fn find(&self, code: &str) -> Option<AdaptiveFlowLogTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveFlowLogTemplate) {
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

    fn list(&self) -> Vec<AdaptiveFlowLogTemplate> {
        let mut out: Vec<AdaptiveFlowLogTemplate> = Vec::new();
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

pub fn new_memory_adaptive_flow_log_template_store() -> Rc<dyn AdaptiveFlowLogTemplateStore> {
    Rc::new(MemoryAdaptiveFlowLogTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveFlowLogTemplateStore {
    pub inner: Rc<dyn AdaptiveFlowLogTemplateStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveFlowLogTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveFlowLogTemplateStore for CachedAdaptiveFlowLogTemplateStore {
    fn find(&self, code: &str) -> Option<AdaptiveFlowLogTemplate> {
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

    fn save(&self, item: &AdaptiveFlowLogTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveFlowLogTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_flow_log_template_store(
    inner: Rc<dyn AdaptiveFlowLogTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveFlowLogTemplateStore> {
    Rc::new(CachedAdaptiveFlowLogTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveFlowLogTemplate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive flow log templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogTemplateQuery {
    pub label: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveFlowLogTemplateQuery {
    pub fn matches(&self, item: &AdaptiveFlowLogTemplate) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveFlowLogTemplate]) -> Vec<AdaptiveFlowLogTemplate> {
        let matched = collections::filter(items, |candidate: &AdaptiveFlowLogTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_flow_log_templates(
    store: Rc<dyn AdaptiveFlowLogTemplateStore>,
    query: &AdaptiveFlowLogTemplateQuery,
) -> Vec<AdaptiveFlowLogTemplate> {
    query.apply(&store.list())
}
