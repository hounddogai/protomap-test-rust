use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dashboard groups by code.
pub trait DashboardGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DashboardGroup>;
    fn save(&self, item: &DashboardGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DashboardGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDashboardGroupStore {
    pub items: RefCell<BTreeMap<String, DashboardGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl DashboardGroupStore for MemoryDashboardGroupStore {
    fn find(&self, code: &str) -> Option<DashboardGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DashboardGroup) {
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

    fn list(&self) -> Vec<DashboardGroup> {
        let mut out: Vec<DashboardGroup> = Vec::new();
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

pub fn new_memory_dashboard_group_store() -> Rc<dyn DashboardGroupStore> {
    Rc::new(MemoryDashboardGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDashboardGroupStore {
    pub inner: Rc<dyn DashboardGroupStore>,
    pub cache: RefCell<cache::TtlCache<DashboardGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DashboardGroupStore for CachedDashboardGroupStore {
    fn find(&self, code: &str) -> Option<DashboardGroup> {
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

    fn save(&self, item: &DashboardGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DashboardGroup> {
        self.inner.list()
    }
}

pub fn new_cached_dashboard_group_store(
    inner: Rc<dyn DashboardGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DashboardGroupStore> {
    Rc::new(CachedDashboardGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DashboardGroup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dashboard groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardGroupQuery {
    pub locale: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DashboardGroupQuery {
    pub fn matches(&self, item: &DashboardGroup) -> bool {
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

    pub fn apply(&self, items: &[DashboardGroup]) -> Vec<DashboardGroup> {
        let matched = collections::filter(items, |candidate: &DashboardGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dashboard_groups(store: Rc<dyn DashboardGroupStore>, query: &DashboardGroupQuery) -> Vec<DashboardGroup> {
    query.apply(&store.list())
}
