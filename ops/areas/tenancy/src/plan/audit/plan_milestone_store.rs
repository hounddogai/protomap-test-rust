use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan milestones by code.
pub trait PlanMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanMilestone>;
    fn save(&self, item: &PlanMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanMilestoneStore {
    pub items: RefCell<BTreeMap<String, PlanMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanMilestoneStore for MemoryPlanMilestoneStore {
    fn find(&self, code: &str) -> Option<PlanMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanMilestone) {
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

    fn list(&self) -> Vec<PlanMilestone> {
        let mut out: Vec<PlanMilestone> = Vec::new();
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

pub fn new_memory_plan_milestone_store() -> Rc<dyn PlanMilestoneStore> {
    Rc::new(MemoryPlanMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanMilestoneStore {
    pub inner: Rc<dyn PlanMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<PlanMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanMilestoneStore for CachedPlanMilestoneStore {
    fn find(&self, code: &str) -> Option<PlanMilestone> {
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

    fn save(&self, item: &PlanMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_plan_milestone_store(
    inner: Rc<dyn PlanMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanMilestoneStore> {
    Rc::new(CachedPlanMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanMilestone>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanMilestoneQuery {
    pub unit: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanMilestoneQuery {
    pub fn matches(&self, item: &PlanMilestone) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[PlanMilestone]) -> Vec<PlanMilestone> {
        let matched = collections::filter(items, |candidate: &PlanMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_milestones(store: Rc<dyn PlanMilestoneStore>, query: &PlanMilestoneQuery) -> Vec<PlanMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PlanMilestoneStore> = new_memory_plan_milestone_store();
        let item = new_plan_milestone("code-841", "label-335");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
