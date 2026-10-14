use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary redaction templates by code.
pub trait PrimaryRedactionTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryRedactionTemplate>;
    fn save(&self, item: &PrimaryRedactionTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryRedactionTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryRedactionTemplateStore {
    pub items: RefCell<BTreeMap<String, PrimaryRedactionTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryRedactionTemplateStore for MemoryPrimaryRedactionTemplateStore {
    fn find(&self, code: &str) -> Option<PrimaryRedactionTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryRedactionTemplate) {
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

    fn list(&self) -> Vec<PrimaryRedactionTemplate> {
        let mut out: Vec<PrimaryRedactionTemplate> = Vec::new();
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

pub fn new_memory_primary_redaction_template_store() -> Rc<dyn PrimaryRedactionTemplateStore> {
    Rc::new(MemoryPrimaryRedactionTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryRedactionTemplateStore {
    pub inner: Rc<dyn PrimaryRedactionTemplateStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryRedactionTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryRedactionTemplateStore for CachedPrimaryRedactionTemplateStore {
    fn find(&self, code: &str) -> Option<PrimaryRedactionTemplate> {
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

    fn save(&self, item: &PrimaryRedactionTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryRedactionTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_primary_redaction_template_store(
    inner: Rc<dyn PrimaryRedactionTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryRedactionTemplateStore> {
    Rc::new(CachedPrimaryRedactionTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryRedactionTemplate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary redaction templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionTemplateQuery {
    pub name: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryRedactionTemplateQuery {
    pub fn matches(&self, item: &PrimaryRedactionTemplate) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[PrimaryRedactionTemplate]) -> Vec<PrimaryRedactionTemplate> {
        let matched = collections::filter(items, |candidate: &PrimaryRedactionTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_redaction_templates(
    store: Rc<dyn PrimaryRedactionTemplateStore>,
    query: &PrimaryRedactionTemplateQuery,
) -> Vec<PrimaryRedactionTemplate> {
    query.apply(&store.list())
}
