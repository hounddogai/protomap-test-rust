use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps segment milestones by code.
pub trait SegmentMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SegmentMilestone>;
    fn save(&self, item: &SegmentMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SegmentMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySegmentMilestoneStore {
    pub items: RefCell<BTreeMap<String, SegmentMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl SegmentMilestoneStore for MemorySegmentMilestoneStore {
    fn find(&self, code: &str) -> Option<SegmentMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SegmentMilestone) {
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

    fn list(&self) -> Vec<SegmentMilestone> {
        let mut out: Vec<SegmentMilestone> = Vec::new();
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

pub fn new_memory_segment_milestone_store() -> Rc<dyn SegmentMilestoneStore> {
    Rc::new(MemorySegmentMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSegmentMilestoneStore {
    pub inner: Rc<dyn SegmentMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<SegmentMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SegmentMilestoneStore for CachedSegmentMilestoneStore {
    fn find(&self, code: &str) -> Option<SegmentMilestone> {
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

    fn save(&self, item: &SegmentMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SegmentMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_segment_milestone_store(
    inner: Rc<dyn SegmentMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SegmentMilestoneStore> {
    Rc::new(CachedSegmentMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SegmentMilestone>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects segment milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMilestoneQuery {
    pub label: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SegmentMilestoneQuery {
    pub fn matches(&self, item: &SegmentMilestone) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SegmentMilestone]) -> Vec<SegmentMilestone> {
        let matched = collections::filter(items, |candidate: &SegmentMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_segment_milestones(
    store: Rc<dyn SegmentMilestoneStore>,
    query: &SegmentMilestoneQuery,
) -> Vec<SegmentMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SegmentMilestoneStore> = new_memory_segment_milestone_store();
        let item = new_segment_milestone("code-383", "label-556");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
