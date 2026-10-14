use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tenant templates by code.
pub trait TenantTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TenantTemplate>;
    fn save(&self, item: &TenantTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TenantTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTenantTemplateStore {
    pub items: RefCell<BTreeMap<String, TenantTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl TenantTemplateStore for MemoryTenantTemplateStore {
    fn find(&self, code: &str) -> Option<TenantTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TenantTemplate) {
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

    fn list(&self) -> Vec<TenantTemplate> {
        let mut out: Vec<TenantTemplate> = Vec::new();
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

pub fn new_memory_tenant_template_store() -> Rc<dyn TenantTemplateStore> {
    Rc::new(MemoryTenantTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTenantTemplateStore {
    pub inner: Rc<dyn TenantTemplateStore>,
    pub cache: RefCell<cache::TtlCache<TenantTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TenantTemplateStore for CachedTenantTemplateStore {
    fn find(&self, code: &str) -> Option<TenantTemplate> {
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

    fn save(&self, item: &TenantTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TenantTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_tenant_template_store(
    inner: Rc<dyn TenantTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TenantTemplateStore> {
    Rc::new(CachedTenantTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TenantTemplate>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tenant templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantTemplateQuery {
    pub channel: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TenantTemplateQuery {
    pub fn matches(&self, item: &TenantTemplate) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TenantTemplate]) -> Vec<TenantTemplate> {
        let matched = collections::filter(items, |candidate: &TenantTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tenant_templates(store: Rc<dyn TenantTemplateStore>, query: &TenantTemplateQuery) -> Vec<TenantTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TenantTemplateStore> = new_memory_tenant_template_store();
        let item = new_tenant_template("code-516", "label-117");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
