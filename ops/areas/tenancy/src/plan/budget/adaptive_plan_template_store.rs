use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive plan templates by code.
pub trait AdaptivePlanTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptivePlanTemplate>;
    fn save(&self, item: &AdaptivePlanTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptivePlanTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptivePlanTemplateStore {
    pub items: RefCell<BTreeMap<String, AdaptivePlanTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptivePlanTemplateStore for MemoryAdaptivePlanTemplateStore {
    fn find(&self, code: &str) -> Option<AdaptivePlanTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptivePlanTemplate) {
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

    fn list(&self) -> Vec<AdaptivePlanTemplate> {
        let mut out: Vec<AdaptivePlanTemplate> = Vec::new();
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

pub fn new_memory_adaptive_plan_template_store() -> Rc<dyn AdaptivePlanTemplateStore> {
    Rc::new(MemoryAdaptivePlanTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptivePlanTemplateStore {
    pub inner: Rc<dyn AdaptivePlanTemplateStore>,
    pub cache: RefCell<cache::TtlCache<AdaptivePlanTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptivePlanTemplateStore for CachedAdaptivePlanTemplateStore {
    fn find(&self, code: &str) -> Option<AdaptivePlanTemplate> {
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

    fn save(&self, item: &AdaptivePlanTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptivePlanTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_plan_template_store(
    inner: Rc<dyn AdaptivePlanTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptivePlanTemplateStore> {
    Rc::new(CachedAdaptivePlanTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptivePlanTemplate>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive plan templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanTemplateQuery {
    pub channel: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptivePlanTemplateQuery {
    pub fn matches(&self, item: &AdaptivePlanTemplate) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptivePlanTemplate]) -> Vec<AdaptivePlanTemplate> {
        let matched = collections::filter(items, |candidate: &AdaptivePlanTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_plan_templates(
    store: Rc<dyn AdaptivePlanTemplateStore>,
    query: &AdaptivePlanTemplateQuery,
) -> Vec<AdaptivePlanTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptivePlanTemplateStore> = new_memory_adaptive_plan_template_store();
        let item = new_adaptive_plan_template("code-516", "title-256");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
