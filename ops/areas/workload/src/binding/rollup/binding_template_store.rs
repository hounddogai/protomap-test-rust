use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding templates by code.
pub trait BindingTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingTemplate>;
    fn save(&self, item: &BindingTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingTemplateStore {
    pub items: RefCell<BTreeMap<String, BindingTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingTemplateStore for MemoryBindingTemplateStore {
    fn find(&self, code: &str) -> Option<BindingTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingTemplate) {
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

    fn list(&self) -> Vec<BindingTemplate> {
        let mut out: Vec<BindingTemplate> = Vec::new();
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

pub fn new_memory_binding_template_store() -> Rc<dyn BindingTemplateStore> {
    Rc::new(MemoryBindingTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingTemplateStore {
    pub inner: Rc<dyn BindingTemplateStore>,
    pub cache: RefCell<cache::TtlCache<BindingTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingTemplateStore for CachedBindingTemplateStore {
    fn find(&self, code: &str) -> Option<BindingTemplate> {
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

    fn save(&self, item: &BindingTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_binding_template_store(
    inner: Rc<dyn BindingTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingTemplateStore> {
    Rc::new(CachedBindingTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingTemplate>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingTemplateQuery {
    pub segment: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingTemplateQuery {
    pub fn matches(&self, item: &BindingTemplate) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[BindingTemplate]) -> Vec<BindingTemplate> {
        let matched = collections::filter(items, |candidate: &BindingTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_templates(
    store: Rc<dyn BindingTemplateStore>,
    query: &BindingTemplateQuery,
) -> Vec<BindingTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingTemplateStore> = new_memory_binding_template_store();
        let item = new_binding_template("code-18", "title-533");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
