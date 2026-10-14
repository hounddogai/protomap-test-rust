use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps delegation templates by code.
pub trait DelegationTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DelegationTemplate>;
    fn save(&self, item: &DelegationTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DelegationTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelegationTemplateStore {
    pub items: RefCell<BTreeMap<String, DelegationTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl DelegationTemplateStore for MemoryDelegationTemplateStore {
    fn find(&self, code: &str) -> Option<DelegationTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DelegationTemplate) {
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

    fn list(&self) -> Vec<DelegationTemplate> {
        let mut out: Vec<DelegationTemplate> = Vec::new();
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

pub fn new_memory_delegation_template_store() -> Rc<dyn DelegationTemplateStore> {
    Rc::new(MemoryDelegationTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDelegationTemplateStore {
    pub inner: Rc<dyn DelegationTemplateStore>,
    pub cache: RefCell<cache::TtlCache<DelegationTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DelegationTemplateStore for CachedDelegationTemplateStore {
    fn find(&self, code: &str) -> Option<DelegationTemplate> {
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

    fn save(&self, item: &DelegationTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DelegationTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_delegation_template_store(
    inner: Rc<dyn DelegationTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DelegationTemplateStore> {
    Rc::new(CachedDelegationTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DelegationTemplate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects delegation templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTemplateQuery {
    pub source: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DelegationTemplateQuery {
    pub fn matches(&self, item: &DelegationTemplate) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[DelegationTemplate]) -> Vec<DelegationTemplate> {
        let matched = collections::filter(items, |candidate: &DelegationTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_delegation_templates(
    store: Rc<dyn DelegationTemplateStore>,
    query: &DelegationTemplateQuery,
) -> Vec<DelegationTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DelegationTemplateStore> = new_memory_delegation_template_store();
        let item = new_delegation_template("code-353", "title-780");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
