use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict health check milestones by code.
pub trait StrictHealthCheckMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictHealthCheckMilestone>;
    fn save(&self, item: &StrictHealthCheckMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictHealthCheckMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictHealthCheckMilestoneStore {
    pub items: RefCell<BTreeMap<String, StrictHealthCheckMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictHealthCheckMilestoneStore for MemoryStrictHealthCheckMilestoneStore {
    fn find(&self, code: &str) -> Option<StrictHealthCheckMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictHealthCheckMilestone) {
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

    fn list(&self) -> Vec<StrictHealthCheckMilestone> {
        let mut out: Vec<StrictHealthCheckMilestone> = Vec::new();
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

pub fn new_memory_strict_health_check_milestone_store() -> Rc<dyn StrictHealthCheckMilestoneStore> {
    Rc::new(MemoryStrictHealthCheckMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictHealthCheckMilestoneStore {
    pub inner: Rc<dyn StrictHealthCheckMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<StrictHealthCheckMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictHealthCheckMilestoneStore for CachedStrictHealthCheckMilestoneStore {
    fn find(&self, code: &str) -> Option<StrictHealthCheckMilestone> {
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

    fn save(&self, item: &StrictHealthCheckMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictHealthCheckMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_strict_health_check_milestone_store(
    inner: Rc<dyn StrictHealthCheckMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictHealthCheckMilestoneStore> {
    Rc::new(CachedStrictHealthCheckMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictHealthCheckMilestone>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict health check milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckMilestoneQuery {
    pub namespace: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictHealthCheckMilestoneQuery {
    pub fn matches(&self, item: &StrictHealthCheckMilestone) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[StrictHealthCheckMilestone]) -> Vec<StrictHealthCheckMilestone> {
        let matched = collections::filter(items, |candidate: &StrictHealthCheckMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_health_check_milestones(
    store: Rc<dyn StrictHealthCheckMilestoneStore>,
    query: &StrictHealthCheckMilestoneQuery,
) -> Vec<StrictHealthCheckMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictHealthCheckMilestoneStore> = new_memory_strict_health_check_milestone_store();
        let item = new_strict_health_check_milestone("code-326", "title-802");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
