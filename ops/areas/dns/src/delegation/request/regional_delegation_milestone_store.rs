use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional delegation milestones by code.
pub trait RegionalDelegationMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalDelegationMilestone>;
    fn save(&self, item: &RegionalDelegationMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalDelegationMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalDelegationMilestoneStore {
    pub items: RefCell<BTreeMap<String, RegionalDelegationMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalDelegationMilestoneStore for MemoryRegionalDelegationMilestoneStore {
    fn find(&self, code: &str) -> Option<RegionalDelegationMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalDelegationMilestone) {
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

    fn list(&self) -> Vec<RegionalDelegationMilestone> {
        let mut out: Vec<RegionalDelegationMilestone> = Vec::new();
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

pub fn new_memory_regional_delegation_milestone_store() -> Rc<dyn RegionalDelegationMilestoneStore> {
    Rc::new(MemoryRegionalDelegationMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalDelegationMilestoneStore {
    pub inner: Rc<dyn RegionalDelegationMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<RegionalDelegationMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalDelegationMilestoneStore for CachedRegionalDelegationMilestoneStore {
    fn find(&self, code: &str) -> Option<RegionalDelegationMilestone> {
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

    fn save(&self, item: &RegionalDelegationMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalDelegationMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_regional_delegation_milestone_store(
    inner: Rc<dyn RegionalDelegationMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalDelegationMilestoneStore> {
    Rc::new(CachedRegionalDelegationMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalDelegationMilestone>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional delegation milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestoneQuery {
    pub name: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalDelegationMilestoneQuery {
    pub fn matches(&self, item: &RegionalDelegationMilestone) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalDelegationMilestone]) -> Vec<RegionalDelegationMilestone> {
        let matched = collections::filter(items, |candidate: &RegionalDelegationMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_delegation_milestones(
    store: Rc<dyn RegionalDelegationMilestoneStore>,
    query: &RegionalDelegationMilestoneQuery,
) -> Vec<RegionalDelegationMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalDelegationMilestoneStore> = new_memory_regional_delegation_milestone_store();
        let item = new_regional_delegation_milestone("code-634", "name-606");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
