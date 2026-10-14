use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy isolation milestones by code.
pub trait LegacyIsolationMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyIsolationMilestone>;
    fn save(&self, item: &LegacyIsolationMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyIsolationMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyIsolationMilestoneStore {
    pub items: RefCell<BTreeMap<String, LegacyIsolationMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyIsolationMilestoneStore for MemoryLegacyIsolationMilestoneStore {
    fn find(&self, code: &str) -> Option<LegacyIsolationMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyIsolationMilestone) {
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

    fn list(&self) -> Vec<LegacyIsolationMilestone> {
        let mut out: Vec<LegacyIsolationMilestone> = Vec::new();
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

pub fn new_memory_legacy_isolation_milestone_store() -> Rc<dyn LegacyIsolationMilestoneStore> {
    Rc::new(MemoryLegacyIsolationMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyIsolationMilestoneStore {
    pub inner: Rc<dyn LegacyIsolationMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<LegacyIsolationMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyIsolationMilestoneStore for CachedLegacyIsolationMilestoneStore {
    fn find(&self, code: &str) -> Option<LegacyIsolationMilestone> {
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

    fn save(&self, item: &LegacyIsolationMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyIsolationMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_isolation_milestone_store(
    inner: Rc<dyn LegacyIsolationMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyIsolationMilestoneStore> {
    Rc::new(CachedLegacyIsolationMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyIsolationMilestone>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy isolation milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationMilestoneQuery {
    pub name: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyIsolationMilestoneQuery {
    pub fn matches(&self, item: &LegacyIsolationMilestone) -> bool {
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

    pub fn apply(&self, items: &[LegacyIsolationMilestone]) -> Vec<LegacyIsolationMilestone> {
        let matched = collections::filter(items, |candidate: &LegacyIsolationMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_isolation_milestones(
    store: Rc<dyn LegacyIsolationMilestoneStore>,
    query: &LegacyIsolationMilestoneQuery,
) -> Vec<LegacyIsolationMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyIsolationMilestoneStore> = new_memory_legacy_isolation_milestone_store();
        let item = new_legacy_isolation_milestone("code-369", "name-983");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
