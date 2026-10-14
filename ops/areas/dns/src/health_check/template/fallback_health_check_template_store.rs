use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback health check templates by code.
pub trait FallbackHealthCheckTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackHealthCheckTemplate>;
    fn save(&self, item: &FallbackHealthCheckTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackHealthCheckTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackHealthCheckTemplateStore {
    pub items: RefCell<BTreeMap<String, FallbackHealthCheckTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackHealthCheckTemplateStore for MemoryFallbackHealthCheckTemplateStore {
    fn find(&self, code: &str) -> Option<FallbackHealthCheckTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackHealthCheckTemplate) {
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

    fn list(&self) -> Vec<FallbackHealthCheckTemplate> {
        let mut out: Vec<FallbackHealthCheckTemplate> = Vec::new();
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

pub fn new_memory_fallback_health_check_template_store() -> Rc<dyn FallbackHealthCheckTemplateStore> {
    Rc::new(MemoryFallbackHealthCheckTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackHealthCheckTemplateStore {
    pub inner: Rc<dyn FallbackHealthCheckTemplateStore>,
    pub cache: RefCell<cache::TtlCache<FallbackHealthCheckTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackHealthCheckTemplateStore for CachedFallbackHealthCheckTemplateStore {
    fn find(&self, code: &str) -> Option<FallbackHealthCheckTemplate> {
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

    fn save(&self, item: &FallbackHealthCheckTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackHealthCheckTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_health_check_template_store(
    inner: Rc<dyn FallbackHealthCheckTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackHealthCheckTemplateStore> {
    Rc::new(CachedFallbackHealthCheckTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackHealthCheckTemplate>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback health check templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckTemplateQuery {
    pub cluster: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackHealthCheckTemplateQuery {
    pub fn matches(&self, item: &FallbackHealthCheckTemplate) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackHealthCheckTemplate]) -> Vec<FallbackHealthCheckTemplate> {
        let matched = collections::filter(items, |candidate: &FallbackHealthCheckTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_health_check_templates(
    store: Rc<dyn FallbackHealthCheckTemplateStore>,
    query: &FallbackHealthCheckTemplateQuery,
) -> Vec<FallbackHealthCheckTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackHealthCheckTemplateStore> = new_memory_fallback_health_check_template_store();
        let item = new_fallback_health_check_template("code-369", "title-983");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
