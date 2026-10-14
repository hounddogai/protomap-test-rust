use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy rollback milestones by code.
pub trait LegacyRollbackMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyRollbackMilestone>;
    fn save(&self, item: &LegacyRollbackMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyRollbackMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyRollbackMilestoneStore {
    pub items: RefCell<BTreeMap<String, LegacyRollbackMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyRollbackMilestoneStore for MemoryLegacyRollbackMilestoneStore {
    fn find(&self, code: &str) -> Option<LegacyRollbackMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyRollbackMilestone) {
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

    fn list(&self) -> Vec<LegacyRollbackMilestone> {
        let mut out: Vec<LegacyRollbackMilestone> = Vec::new();
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

pub fn new_memory_legacy_rollback_milestone_store() -> Rc<dyn LegacyRollbackMilestoneStore> {
    Rc::new(MemoryLegacyRollbackMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyRollbackMilestoneStore {
    pub inner: Rc<dyn LegacyRollbackMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<LegacyRollbackMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyRollbackMilestoneStore for CachedLegacyRollbackMilestoneStore {
    fn find(&self, code: &str) -> Option<LegacyRollbackMilestone> {
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

    fn save(&self, item: &LegacyRollbackMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyRollbackMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_rollback_milestone_store(
    inner: Rc<dyn LegacyRollbackMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyRollbackMilestoneStore> {
    Rc::new(CachedLegacyRollbackMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyRollbackMilestone>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy rollback milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMilestoneQuery {
    pub name: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyRollbackMilestoneQuery {
    pub fn matches(&self, item: &LegacyRollbackMilestone) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyRollbackMilestone]) -> Vec<LegacyRollbackMilestone> {
        let matched = collections::filter(items, |candidate: &LegacyRollbackMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_rollback_milestones(
    store: Rc<dyn LegacyRollbackMilestoneStore>,
    query: &LegacyRollbackMilestoneQuery,
) -> Vec<LegacyRollbackMilestone> {
    query.apply(&store.list())
}
