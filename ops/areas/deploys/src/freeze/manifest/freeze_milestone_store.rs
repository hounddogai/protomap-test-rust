use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps freeze milestones by code.
pub trait FreezeMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FreezeMilestone>;
    fn save(&self, item: &FreezeMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FreezeMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFreezeMilestoneStore {
    pub items: RefCell<BTreeMap<String, FreezeMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl FreezeMilestoneStore for MemoryFreezeMilestoneStore {
    fn find(&self, code: &str) -> Option<FreezeMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FreezeMilestone) {
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

    fn list(&self) -> Vec<FreezeMilestone> {
        let mut out: Vec<FreezeMilestone> = Vec::new();
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

pub fn new_memory_freeze_milestone_store() -> Rc<dyn FreezeMilestoneStore> {
    Rc::new(MemoryFreezeMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFreezeMilestoneStore {
    pub inner: Rc<dyn FreezeMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<FreezeMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FreezeMilestoneStore for CachedFreezeMilestoneStore {
    fn find(&self, code: &str) -> Option<FreezeMilestone> {
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

    fn save(&self, item: &FreezeMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FreezeMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_freeze_milestone_store(
    inner: Rc<dyn FreezeMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FreezeMilestoneStore> {
    Rc::new(CachedFreezeMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FreezeMilestone>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects freeze milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMilestoneQuery {
    pub country: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FreezeMilestoneQuery {
    pub fn matches(&self, item: &FreezeMilestone) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FreezeMilestone]) -> Vec<FreezeMilestone> {
        let matched = collections::filter(items, |candidate: &FreezeMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_freeze_milestones(
    store: Rc<dyn FreezeMilestoneStore>,
    query: &FreezeMilestoneQuery,
) -> Vec<FreezeMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FreezeMilestoneStore> = new_memory_freeze_milestone_store();
        let item = new_freeze_milestone("code-762", "label-138");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
