use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps isolation templates by code.
pub trait IsolationTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IsolationTemplate>;
    fn save(&self, item: &IsolationTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IsolationTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIsolationTemplateStore {
    pub items: RefCell<BTreeMap<String, IsolationTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl IsolationTemplateStore for MemoryIsolationTemplateStore {
    fn find(&self, code: &str) -> Option<IsolationTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IsolationTemplate) {
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

    fn list(&self) -> Vec<IsolationTemplate> {
        let mut out: Vec<IsolationTemplate> = Vec::new();
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

pub fn new_memory_isolation_template_store() -> Rc<dyn IsolationTemplateStore> {
    Rc::new(MemoryIsolationTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIsolationTemplateStore {
    pub inner: Rc<dyn IsolationTemplateStore>,
    pub cache: RefCell<cache::TtlCache<IsolationTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IsolationTemplateStore for CachedIsolationTemplateStore {
    fn find(&self, code: &str) -> Option<IsolationTemplate> {
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

    fn save(&self, item: &IsolationTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IsolationTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_isolation_template_store(
    inner: Rc<dyn IsolationTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IsolationTemplateStore> {
    Rc::new(CachedIsolationTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IsolationTemplate>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects isolation templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTemplateQuery {
    pub title: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IsolationTemplateQuery {
    pub fn matches(&self, item: &IsolationTemplate) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IsolationTemplate]) -> Vec<IsolationTemplate> {
        let matched = collections::filter(items, |candidate: &IsolationTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_isolation_templates(
    store: Rc<dyn IsolationTemplateStore>,
    query: &IsolationTemplateQuery,
) -> Vec<IsolationTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IsolationTemplateStore> = new_memory_isolation_template_store();
        let item = new_isolation_template("code-892", "title-843");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
