use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual budget burn templates by code.
pub trait ManualBudgetBurnTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualBudgetBurnTemplate>;
    fn save(&self, item: &ManualBudgetBurnTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualBudgetBurnTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualBudgetBurnTemplateStore {
    pub items: RefCell<BTreeMap<String, ManualBudgetBurnTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualBudgetBurnTemplateStore for MemoryManualBudgetBurnTemplateStore {
    fn find(&self, code: &str) -> Option<ManualBudgetBurnTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualBudgetBurnTemplate) {
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

    fn list(&self) -> Vec<ManualBudgetBurnTemplate> {
        let mut out: Vec<ManualBudgetBurnTemplate> = Vec::new();
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

pub fn new_memory_manual_budget_burn_template_store() -> Rc<dyn ManualBudgetBurnTemplateStore> {
    Rc::new(MemoryManualBudgetBurnTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualBudgetBurnTemplateStore {
    pub inner: Rc<dyn ManualBudgetBurnTemplateStore>,
    pub cache: RefCell<cache::TtlCache<ManualBudgetBurnTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualBudgetBurnTemplateStore for CachedManualBudgetBurnTemplateStore {
    fn find(&self, code: &str) -> Option<ManualBudgetBurnTemplate> {
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

    fn save(&self, item: &ManualBudgetBurnTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualBudgetBurnTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_manual_budget_burn_template_store(
    inner: Rc<dyn ManualBudgetBurnTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualBudgetBurnTemplateStore> {
    Rc::new(CachedManualBudgetBurnTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualBudgetBurnTemplate>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual budget burn templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnTemplateQuery {
    pub cluster: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualBudgetBurnTemplateQuery {
    pub fn matches(&self, item: &ManualBudgetBurnTemplate) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualBudgetBurnTemplate]) -> Vec<ManualBudgetBurnTemplate> {
        let matched = collections::filter(items, |candidate: &ManualBudgetBurnTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_budget_burn_templates(
    store: Rc<dyn ManualBudgetBurnTemplateStore>,
    query: &ManualBudgetBurnTemplateQuery,
) -> Vec<ManualBudgetBurnTemplate> {
    query.apply(&store.list())
}
