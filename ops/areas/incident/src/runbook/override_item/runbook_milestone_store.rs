use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps runbook milestones by code.
pub trait RunbookMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RunbookMilestone>;
    fn save(&self, item: &RunbookMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RunbookMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRunbookMilestoneStore {
    pub items: RefCell<BTreeMap<String, RunbookMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl RunbookMilestoneStore for MemoryRunbookMilestoneStore {
    fn find(&self, code: &str) -> Option<RunbookMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RunbookMilestone) {
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

    fn list(&self) -> Vec<RunbookMilestone> {
        let mut out: Vec<RunbookMilestone> = Vec::new();
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

pub fn new_memory_runbook_milestone_store() -> Rc<dyn RunbookMilestoneStore> {
    Rc::new(MemoryRunbookMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRunbookMilestoneStore {
    pub inner: Rc<dyn RunbookMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<RunbookMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RunbookMilestoneStore for CachedRunbookMilestoneStore {
    fn find(&self, code: &str) -> Option<RunbookMilestone> {
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

    fn save(&self, item: &RunbookMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RunbookMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_runbook_milestone_store(
    inner: Rc<dyn RunbookMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RunbookMilestoneStore> {
    Rc::new(CachedRunbookMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RunbookMilestone>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects runbook milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMilestoneQuery {
    pub source: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RunbookMilestoneQuery {
    pub fn matches(&self, item: &RunbookMilestone) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[RunbookMilestone]) -> Vec<RunbookMilestone> {
        let matched = collections::filter(items, |candidate: &RunbookMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_runbook_milestones(
    store: Rc<dyn RunbookMilestoneStore>,
    query: &RunbookMilestoneQuery,
) -> Vec<RunbookMilestone> {
    query.apply(&store.list())
}
