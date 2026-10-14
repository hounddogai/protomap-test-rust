use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction templates by code.
pub trait RedactionTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionTemplate>;
    fn save(&self, item: &RedactionTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionTemplateStore {
    pub items: RefCell<BTreeMap<String, RedactionTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionTemplateStore for MemoryRedactionTemplateStore {
    fn find(&self, code: &str) -> Option<RedactionTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionTemplate) {
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

    fn list(&self) -> Vec<RedactionTemplate> {
        let mut out: Vec<RedactionTemplate> = Vec::new();
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

pub fn new_memory_redaction_template_store() -> Rc<dyn RedactionTemplateStore> {
    Rc::new(MemoryRedactionTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionTemplateStore {
    pub inner: Rc<dyn RedactionTemplateStore>,
    pub cache: RefCell<cache::TtlCache<RedactionTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionTemplateStore for CachedRedactionTemplateStore {
    fn find(&self, code: &str) -> Option<RedactionTemplate> {
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

    fn save(&self, item: &RedactionTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_template_store(
    inner: Rc<dyn RedactionTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionTemplateStore> {
    Rc::new(CachedRedactionTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionTemplate>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTemplateQuery {
    pub locale: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionTemplateQuery {
    pub fn matches(&self, item: &RedactionTemplate) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RedactionTemplate]) -> Vec<RedactionTemplate> {
        let matched = collections::filter(items, |candidate: &RedactionTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_templates(
    store: Rc<dyn RedactionTemplateStore>,
    query: &RedactionTemplateQuery,
) -> Vec<RedactionTemplate> {
    query.apply(&store.list())
}
