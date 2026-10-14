use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps delegation milestones by code.
pub trait DelegationMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DelegationMilestone>;
    fn save(&self, item: &DelegationMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DelegationMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelegationMilestoneStore {
    pub items: RefCell<BTreeMap<String, DelegationMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl DelegationMilestoneStore for MemoryDelegationMilestoneStore {
    fn find(&self, code: &str) -> Option<DelegationMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DelegationMilestone) {
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

    fn list(&self) -> Vec<DelegationMilestone> {
        let mut out: Vec<DelegationMilestone> = Vec::new();
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

pub fn new_memory_delegation_milestone_store() -> Rc<dyn DelegationMilestoneStore> {
    Rc::new(MemoryDelegationMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDelegationMilestoneStore {
    pub inner: Rc<dyn DelegationMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<DelegationMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DelegationMilestoneStore for CachedDelegationMilestoneStore {
    fn find(&self, code: &str) -> Option<DelegationMilestone> {
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

    fn save(&self, item: &DelegationMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DelegationMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_delegation_milestone_store(
    inner: Rc<dyn DelegationMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DelegationMilestoneStore> {
    Rc::new(CachedDelegationMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DelegationMilestone>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects delegation milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMilestoneQuery {
    pub name: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DelegationMilestoneQuery {
    pub fn matches(&self, item: &DelegationMilestone) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DelegationMilestone]) -> Vec<DelegationMilestone> {
        let matched = collections::filter(items, |candidate: &DelegationMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_delegation_milestones(
    store: Rc<dyn DelegationMilestoneStore>,
    query: &DelegationMilestoneQuery,
) -> Vec<DelegationMilestone> {
    query.apply(&store.list())
}
