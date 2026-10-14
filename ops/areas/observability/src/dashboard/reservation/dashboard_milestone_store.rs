use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dashboard milestones by code.
pub trait DashboardMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DashboardMilestone>;
    fn save(&self, item: &DashboardMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DashboardMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDashboardMilestoneStore {
    pub items: RefCell<BTreeMap<String, DashboardMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl DashboardMilestoneStore for MemoryDashboardMilestoneStore {
    fn find(&self, code: &str) -> Option<DashboardMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DashboardMilestone) {
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

    fn list(&self) -> Vec<DashboardMilestone> {
        let mut out: Vec<DashboardMilestone> = Vec::new();
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

pub fn new_memory_dashboard_milestone_store() -> Rc<dyn DashboardMilestoneStore> {
    Rc::new(MemoryDashboardMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDashboardMilestoneStore {
    pub inner: Rc<dyn DashboardMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<DashboardMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DashboardMilestoneStore for CachedDashboardMilestoneStore {
    fn find(&self, code: &str) -> Option<DashboardMilestone> {
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

    fn save(&self, item: &DashboardMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DashboardMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_dashboard_milestone_store(
    inner: Rc<dyn DashboardMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DashboardMilestoneStore> {
    Rc::new(CachedDashboardMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DashboardMilestone>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dashboard milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMilestoneQuery {
    pub source: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DashboardMilestoneQuery {
    pub fn matches(&self, item: &DashboardMilestone) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DashboardMilestone]) -> Vec<DashboardMilestone> {
        let matched = collections::filter(items, |candidate: &DashboardMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dashboard_milestones(
    store: Rc<dyn DashboardMilestoneStore>,
    query: &DashboardMilestoneQuery,
) -> Vec<DashboardMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DashboardMilestoneStore> = new_memory_dashboard_milestone_store();
        let item = new_dashboard_milestone("code-338", "name-366");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
