use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default dashboard estimates by code.
pub trait DefaultDashboardEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultDashboardEstimate>;
    fn save(&self, item: &DefaultDashboardEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultDashboardEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultDashboardEstimateStore {
    pub items: RefCell<BTreeMap<String, DefaultDashboardEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultDashboardEstimateStore for MemoryDefaultDashboardEstimateStore {
    fn find(&self, code: &str) -> Option<DefaultDashboardEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultDashboardEstimate) {
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

    fn list(&self) -> Vec<DefaultDashboardEstimate> {
        let mut out: Vec<DefaultDashboardEstimate> = Vec::new();
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

pub fn new_memory_default_dashboard_estimate_store() -> Rc<dyn DefaultDashboardEstimateStore> {
    Rc::new(MemoryDefaultDashboardEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultDashboardEstimateStore {
    pub inner: Rc<dyn DefaultDashboardEstimateStore>,
    pub cache: RefCell<cache::TtlCache<DefaultDashboardEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultDashboardEstimateStore for CachedDefaultDashboardEstimateStore {
    fn find(&self, code: &str) -> Option<DefaultDashboardEstimate> {
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

    fn save(&self, item: &DefaultDashboardEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultDashboardEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_default_dashboard_estimate_store(
    inner: Rc<dyn DefaultDashboardEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultDashboardEstimateStore> {
    Rc::new(CachedDefaultDashboardEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultDashboardEstimate>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default dashboard estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardEstimateQuery {
    pub title: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultDashboardEstimateQuery {
    pub fn matches(&self, item: &DefaultDashboardEstimate) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultDashboardEstimate]) -> Vec<DefaultDashboardEstimate> {
        let matched = collections::filter(items, |candidate: &DefaultDashboardEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_dashboard_estimates(
    store: Rc<dyn DefaultDashboardEstimateStore>,
    query: &DefaultDashboardEstimateQuery,
) -> Vec<DefaultDashboardEstimate> {
    query.apply(&store.list())
}
