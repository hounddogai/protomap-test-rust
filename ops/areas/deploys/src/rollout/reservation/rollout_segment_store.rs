use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollout segments by code.
pub trait RolloutSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RolloutSegment>;
    fn save(&self, item: &RolloutSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RolloutSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRolloutSegmentStore {
    pub items: RefCell<BTreeMap<String, RolloutSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl RolloutSegmentStore for MemoryRolloutSegmentStore {
    fn find(&self, code: &str) -> Option<RolloutSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RolloutSegment) {
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

    fn list(&self) -> Vec<RolloutSegment> {
        let mut out: Vec<RolloutSegment> = Vec::new();
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

pub fn new_memory_rollout_segment_store() -> Rc<dyn RolloutSegmentStore> {
    Rc::new(MemoryRolloutSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRolloutSegmentStore {
    pub inner: Rc<dyn RolloutSegmentStore>,
    pub cache: RefCell<cache::TtlCache<RolloutSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RolloutSegmentStore for CachedRolloutSegmentStore {
    fn find(&self, code: &str) -> Option<RolloutSegment> {
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

    fn save(&self, item: &RolloutSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RolloutSegment> {
        self.inner.list()
    }
}

pub fn new_cached_rollout_segment_store(
    inner: Rc<dyn RolloutSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RolloutSegmentStore> {
    Rc::new(CachedRolloutSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RolloutSegment>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollout segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSegmentQuery {
    pub country: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RolloutSegmentQuery {
    pub fn matches(&self, item: &RolloutSegment) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RolloutSegment]) -> Vec<RolloutSegment> {
        let matched = collections::filter(items, |candidate: &RolloutSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollout_segments(store: Rc<dyn RolloutSegmentStore>, query: &RolloutSegmentQuery) -> Vec<RolloutSegment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RolloutSegmentStore> = new_memory_rollout_segment_store();
        let item = new_rollout_segment("code-749", "title-690");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
