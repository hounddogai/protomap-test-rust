use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling identity milestones by code.
pub trait RollingIdentityMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingIdentityMilestone>;
    fn save(&self, item: &RollingIdentityMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingIdentityMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingIdentityMilestoneStore {
    pub items: RefCell<BTreeMap<String, RollingIdentityMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingIdentityMilestoneStore for MemoryRollingIdentityMilestoneStore {
    fn find(&self, code: &str) -> Option<RollingIdentityMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingIdentityMilestone) {
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

    fn list(&self) -> Vec<RollingIdentityMilestone> {
        let mut out: Vec<RollingIdentityMilestone> = Vec::new();
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

pub fn new_memory_rolling_identity_milestone_store() -> Rc<dyn RollingIdentityMilestoneStore> {
    Rc::new(MemoryRollingIdentityMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingIdentityMilestoneStore {
    pub inner: Rc<dyn RollingIdentityMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<RollingIdentityMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingIdentityMilestoneStore for CachedRollingIdentityMilestoneStore {
    fn find(&self, code: &str) -> Option<RollingIdentityMilestone> {
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

    fn save(&self, item: &RollingIdentityMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingIdentityMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_identity_milestone_store(
    inner: Rc<dyn RollingIdentityMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingIdentityMilestoneStore> {
    Rc::new(CachedRollingIdentityMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingIdentityMilestone>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling identity milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityMilestoneQuery {
    pub name: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingIdentityMilestoneQuery {
    pub fn matches(&self, item: &RollingIdentityMilestone) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingIdentityMilestone]) -> Vec<RollingIdentityMilestone> {
        let matched = collections::filter(items, |candidate: &RollingIdentityMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_identity_milestones(
    store: Rc<dyn RollingIdentityMilestoneStore>,
    query: &RollingIdentityMilestoneQuery,
) -> Vec<RollingIdentityMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingIdentityMilestoneStore> = new_memory_rolling_identity_milestone_store();
        let item = new_rolling_identity_milestone("code-4", "name-760");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
