use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict rollout segments by code.
pub trait StrictRolloutSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictRolloutSegment>;
    fn save(&self, item: &StrictRolloutSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictRolloutSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictRolloutSegmentStore {
    pub items: RefCell<BTreeMap<String, StrictRolloutSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictRolloutSegmentStore for MemoryStrictRolloutSegmentStore {
    fn find(&self, code: &str) -> Option<StrictRolloutSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictRolloutSegment) {
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

    fn list(&self) -> Vec<StrictRolloutSegment> {
        let mut out: Vec<StrictRolloutSegment> = Vec::new();
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

pub fn new_memory_strict_rollout_segment_store() -> Rc<dyn StrictRolloutSegmentStore> {
    Rc::new(MemoryStrictRolloutSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictRolloutSegmentStore {
    pub inner: Rc<dyn StrictRolloutSegmentStore>,
    pub cache: RefCell<cache::TtlCache<StrictRolloutSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictRolloutSegmentStore for CachedStrictRolloutSegmentStore {
    fn find(&self, code: &str) -> Option<StrictRolloutSegment> {
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

    fn save(&self, item: &StrictRolloutSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictRolloutSegment> {
        self.inner.list()
    }
}

pub fn new_cached_strict_rollout_segment_store(
    inner: Rc<dyn StrictRolloutSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictRolloutSegmentStore> {
    Rc::new(CachedStrictRolloutSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictRolloutSegment>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict rollout segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegmentQuery {
    pub name: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictRolloutSegmentQuery {
    pub fn matches(&self, item: &StrictRolloutSegment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictRolloutSegment]) -> Vec<StrictRolloutSegment> {
        let matched = collections::filter(items, |candidate: &StrictRolloutSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_rollout_segments(
    store: Rc<dyn StrictRolloutSegmentStore>,
    query: &StrictRolloutSegmentQuery,
) -> Vec<StrictRolloutSegment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictRolloutSegmentStore> = new_memory_strict_rollout_segment_store();
        let item = new_strict_rollout_segment("code-893", "name-743");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
