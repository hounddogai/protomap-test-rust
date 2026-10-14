use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tenant milestones by code.
pub trait TenantMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TenantMilestone>;
    fn save(&self, item: &TenantMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TenantMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTenantMilestoneStore {
    pub items: RefCell<BTreeMap<String, TenantMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl TenantMilestoneStore for MemoryTenantMilestoneStore {
    fn find(&self, code: &str) -> Option<TenantMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TenantMilestone) {
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

    fn list(&self) -> Vec<TenantMilestone> {
        let mut out: Vec<TenantMilestone> = Vec::new();
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

pub fn new_memory_tenant_milestone_store() -> Rc<dyn TenantMilestoneStore> {
    Rc::new(MemoryTenantMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTenantMilestoneStore {
    pub inner: Rc<dyn TenantMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<TenantMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TenantMilestoneStore for CachedTenantMilestoneStore {
    fn find(&self, code: &str) -> Option<TenantMilestone> {
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

    fn save(&self, item: &TenantMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TenantMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_tenant_milestone_store(
    inner: Rc<dyn TenantMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TenantMilestoneStore> {
    Rc::new(CachedTenantMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TenantMilestone>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tenant milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantMilestoneQuery {
    pub label: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TenantMilestoneQuery {
    pub fn matches(&self, item: &TenantMilestone) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TenantMilestone]) -> Vec<TenantMilestone> {
        let matched = collections::filter(items, |candidate: &TenantMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tenant_milestones(
    store: Rc<dyn TenantMilestoneStore>,
    query: &TenantMilestoneQuery,
) -> Vec<TenantMilestone> {
    query.apply(&store.list())
}
