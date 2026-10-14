use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dashboard templates by code.
pub trait DashboardTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DashboardTemplate>;
    fn save(&self, item: &DashboardTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DashboardTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDashboardTemplateStore {
    pub items: RefCell<BTreeMap<String, DashboardTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl DashboardTemplateStore for MemoryDashboardTemplateStore {
    fn find(&self, code: &str) -> Option<DashboardTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DashboardTemplate) {
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

    fn list(&self) -> Vec<DashboardTemplate> {
        let mut out: Vec<DashboardTemplate> = Vec::new();
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

pub fn new_memory_dashboard_template_store() -> Rc<dyn DashboardTemplateStore> {
    Rc::new(MemoryDashboardTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDashboardTemplateStore {
    pub inner: Rc<dyn DashboardTemplateStore>,
    pub cache: RefCell<cache::TtlCache<DashboardTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DashboardTemplateStore for CachedDashboardTemplateStore {
    fn find(&self, code: &str) -> Option<DashboardTemplate> {
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

    fn save(&self, item: &DashboardTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DashboardTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_dashboard_template_store(
    inner: Rc<dyn DashboardTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DashboardTemplateStore> {
    Rc::new(CachedDashboardTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DashboardTemplate>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dashboard templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTemplateQuery {
    pub scope: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DashboardTemplateQuery {
    pub fn matches(&self, item: &DashboardTemplate) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DashboardTemplate]) -> Vec<DashboardTemplate> {
        let matched = collections::filter(items, |candidate: &DashboardTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dashboard_templates(
    store: Rc<dyn DashboardTemplateStore>,
    query: &DashboardTemplateQuery,
) -> Vec<DashboardTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DashboardTemplateStore> = new_memory_dashboard_template_store();
        let item = new_dashboard_template("code-19", "name-453");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
