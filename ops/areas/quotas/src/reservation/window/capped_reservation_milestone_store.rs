use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped reservation milestones by code.
pub trait CappedReservationMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedReservationMilestone>;
    fn save(&self, item: &CappedReservationMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedReservationMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedReservationMilestoneStore {
    pub items: RefCell<BTreeMap<String, CappedReservationMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedReservationMilestoneStore for MemoryCappedReservationMilestoneStore {
    fn find(&self, code: &str) -> Option<CappedReservationMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedReservationMilestone) {
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

    fn list(&self) -> Vec<CappedReservationMilestone> {
        let mut out: Vec<CappedReservationMilestone> = Vec::new();
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

pub fn new_memory_capped_reservation_milestone_store() -> Rc<dyn CappedReservationMilestoneStore> {
    Rc::new(MemoryCappedReservationMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedReservationMilestoneStore {
    pub inner: Rc<dyn CappedReservationMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<CappedReservationMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedReservationMilestoneStore for CachedCappedReservationMilestoneStore {
    fn find(&self, code: &str) -> Option<CappedReservationMilestone> {
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

    fn save(&self, item: &CappedReservationMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedReservationMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_capped_reservation_milestone_store(
    inner: Rc<dyn CappedReservationMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedReservationMilestoneStore> {
    Rc::new(CachedCappedReservationMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedReservationMilestone>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped reservation milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationMilestoneQuery {
    pub name: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedReservationMilestoneQuery {
    pub fn matches(&self, item: &CappedReservationMilestone) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedReservationMilestone]) -> Vec<CappedReservationMilestone> {
        let matched = collections::filter(items, |candidate: &CappedReservationMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_reservation_milestones(
    store: Rc<dyn CappedReservationMilestoneStore>,
    query: &CappedReservationMilestoneQuery,
) -> Vec<CappedReservationMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedReservationMilestoneStore> = new_memory_capped_reservation_milestone_store();
        let item = new_capped_reservation_milestone("code-207", "name-220");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
