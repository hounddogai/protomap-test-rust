use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps nonce templates by code.
pub trait NonceTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<NonceTemplate>;
    fn save(&self, item: &NonceTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<NonceTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNonceTemplateStore {
    pub items: RefCell<BTreeMap<String, NonceTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl NonceTemplateStore for MemoryNonceTemplateStore {
    fn find(&self, code: &str) -> Option<NonceTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &NonceTemplate) {
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

    fn list(&self) -> Vec<NonceTemplate> {
        let mut out: Vec<NonceTemplate> = Vec::new();
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

pub fn new_memory_nonce_template_store() -> Rc<dyn NonceTemplateStore> {
    Rc::new(MemoryNonceTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedNonceTemplateStore {
    pub inner: Rc<dyn NonceTemplateStore>,
    pub cache: RefCell<cache::TtlCache<NonceTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl NonceTemplateStore for CachedNonceTemplateStore {
    fn find(&self, code: &str) -> Option<NonceTemplate> {
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

    fn save(&self, item: &NonceTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<NonceTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_nonce_template_store(
    inner: Rc<dyn NonceTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn NonceTemplateStore> {
    Rc::new(CachedNonceTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<NonceTemplate>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects nonce templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceTemplateQuery {
    pub name: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl NonceTemplateQuery {
    pub fn matches(&self, item: &NonceTemplate) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[NonceTemplate]) -> Vec<NonceTemplate> {
        let matched = collections::filter(items, |candidate: &NonceTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_nonce_templates(store: Rc<dyn NonceTemplateStore>, query: &NonceTemplateQuery) -> Vec<NonceTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn NonceTemplateStore> = new_memory_nonce_template_store();
        let item = new_nonce_template("code-814", "name-276");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
