use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps health check templates by code.
pub trait HealthCheckTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<HealthCheckTemplate>;
    fn save(&self, item: &HealthCheckTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<HealthCheckTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHealthCheckTemplateStore {
    pub items: RefCell<BTreeMap<String, HealthCheckTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl HealthCheckTemplateStore for MemoryHealthCheckTemplateStore {
    fn find(&self, code: &str) -> Option<HealthCheckTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &HealthCheckTemplate) {
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

    fn list(&self) -> Vec<HealthCheckTemplate> {
        let mut out: Vec<HealthCheckTemplate> = Vec::new();
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

pub fn new_memory_health_check_template_store() -> Rc<dyn HealthCheckTemplateStore> {
    Rc::new(MemoryHealthCheckTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedHealthCheckTemplateStore {
    pub inner: Rc<dyn HealthCheckTemplateStore>,
    pub cache: RefCell<cache::TtlCache<HealthCheckTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl HealthCheckTemplateStore for CachedHealthCheckTemplateStore {
    fn find(&self, code: &str) -> Option<HealthCheckTemplate> {
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

    fn save(&self, item: &HealthCheckTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<HealthCheckTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_health_check_template_store(
    inner: Rc<dyn HealthCheckTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn HealthCheckTemplateStore> {
    Rc::new(CachedHealthCheckTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<HealthCheckTemplate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects health check templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTemplateQuery {
    pub channel: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl HealthCheckTemplateQuery {
    pub fn matches(&self, item: &HealthCheckTemplate) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[HealthCheckTemplate]) -> Vec<HealthCheckTemplate> {
        let matched = collections::filter(items, |candidate: &HealthCheckTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_health_check_templates(
    store: Rc<dyn HealthCheckTemplateStore>,
    query: &HealthCheckTemplateQuery,
) -> Vec<HealthCheckTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn HealthCheckTemplateStore> = new_memory_health_check_template_store();
        let item = new_health_check_template("code-282", "label-227");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
