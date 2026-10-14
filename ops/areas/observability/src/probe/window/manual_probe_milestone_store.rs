use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual probe milestones by code.
pub trait ManualProbeMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualProbeMilestone>;
    fn save(&self, item: &ManualProbeMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualProbeMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualProbeMilestoneStore {
    pub items: RefCell<BTreeMap<String, ManualProbeMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualProbeMilestoneStore for MemoryManualProbeMilestoneStore {
    fn find(&self, code: &str) -> Option<ManualProbeMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualProbeMilestone) {
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

    fn list(&self) -> Vec<ManualProbeMilestone> {
        let mut out: Vec<ManualProbeMilestone> = Vec::new();
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

pub fn new_memory_manual_probe_milestone_store() -> Rc<dyn ManualProbeMilestoneStore> {
    Rc::new(MemoryManualProbeMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualProbeMilestoneStore {
    pub inner: Rc<dyn ManualProbeMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<ManualProbeMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualProbeMilestoneStore for CachedManualProbeMilestoneStore {
    fn find(&self, code: &str) -> Option<ManualProbeMilestone> {
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

    fn save(&self, item: &ManualProbeMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualProbeMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_manual_probe_milestone_store(
    inner: Rc<dyn ManualProbeMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualProbeMilestoneStore> {
    Rc::new(CachedManualProbeMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualProbeMilestone>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual probe milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeMilestoneQuery {
    pub title: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualProbeMilestoneQuery {
    pub fn matches(&self, item: &ManualProbeMilestone) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualProbeMilestone]) -> Vec<ManualProbeMilestone> {
        let matched = collections::filter(items, |candidate: &ManualProbeMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_probe_milestones(
    store: Rc<dyn ManualProbeMilestoneStore>,
    query: &ManualProbeMilestoneQuery,
) -> Vec<ManualProbeMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualProbeMilestoneStore> = new_memory_manual_probe_milestone_store();
        let item = new_manual_probe_milestone("code-220", "title-422");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
