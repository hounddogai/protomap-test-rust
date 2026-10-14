use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps verifier templates by code.
pub trait VerifierTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<VerifierTemplate>;
    fn save(&self, item: &VerifierTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<VerifierTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryVerifierTemplateStore {
    pub items: RefCell<BTreeMap<String, VerifierTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl VerifierTemplateStore for MemoryVerifierTemplateStore {
    fn find(&self, code: &str) -> Option<VerifierTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &VerifierTemplate) {
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

    fn list(&self) -> Vec<VerifierTemplate> {
        let mut out: Vec<VerifierTemplate> = Vec::new();
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

pub fn new_memory_verifier_template_store() -> Rc<dyn VerifierTemplateStore> {
    Rc::new(MemoryVerifierTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedVerifierTemplateStore {
    pub inner: Rc<dyn VerifierTemplateStore>,
    pub cache: RefCell<cache::TtlCache<VerifierTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl VerifierTemplateStore for CachedVerifierTemplateStore {
    fn find(&self, code: &str) -> Option<VerifierTemplate> {
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

    fn save(&self, item: &VerifierTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<VerifierTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_verifier_template_store(
    inner: Rc<dyn VerifierTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn VerifierTemplateStore> {
    Rc::new(CachedVerifierTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<VerifierTemplate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects verifier templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTemplateQuery {
    pub title: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl VerifierTemplateQuery {
    pub fn matches(&self, item: &VerifierTemplate) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[VerifierTemplate]) -> Vec<VerifierTemplate> {
        let matched = collections::filter(items, |candidate: &VerifierTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_verifier_templates(
    store: Rc<dyn VerifierTemplateStore>,
    query: &VerifierTemplateQuery,
) -> Vec<VerifierTemplate> {
    query.apply(&store.list())
}
