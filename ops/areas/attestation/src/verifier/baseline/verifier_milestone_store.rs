use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps verifier milestones by code.
pub trait VerifierMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<VerifierMilestone>;
    fn save(&self, item: &VerifierMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<VerifierMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryVerifierMilestoneStore {
    pub items: RefCell<BTreeMap<String, VerifierMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl VerifierMilestoneStore for MemoryVerifierMilestoneStore {
    fn find(&self, code: &str) -> Option<VerifierMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &VerifierMilestone) {
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

    fn list(&self) -> Vec<VerifierMilestone> {
        let mut out: Vec<VerifierMilestone> = Vec::new();
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

pub fn new_memory_verifier_milestone_store() -> Rc<dyn VerifierMilestoneStore> {
    Rc::new(MemoryVerifierMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedVerifierMilestoneStore {
    pub inner: Rc<dyn VerifierMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<VerifierMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl VerifierMilestoneStore for CachedVerifierMilestoneStore {
    fn find(&self, code: &str) -> Option<VerifierMilestone> {
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

    fn save(&self, item: &VerifierMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<VerifierMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_verifier_milestone_store(
    inner: Rc<dyn VerifierMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn VerifierMilestoneStore> {
    Rc::new(CachedVerifierMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<VerifierMilestone>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects verifier milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMilestoneQuery {
    pub title: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl VerifierMilestoneQuery {
    pub fn matches(&self, item: &VerifierMilestone) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[VerifierMilestone]) -> Vec<VerifierMilestone> {
        let matched = collections::filter(items, |candidate: &VerifierMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_verifier_milestones(
    store: Rc<dyn VerifierMilestoneStore>,
    query: &VerifierMilestoneQuery,
) -> Vec<VerifierMilestone> {
    query.apply(&store.list())
}
