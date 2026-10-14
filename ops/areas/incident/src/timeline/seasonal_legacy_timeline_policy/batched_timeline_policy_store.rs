use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched timeline policies by code.
pub trait BatchedTimelinePolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedTimelinePolicy>;
    fn save(&self, item: &BatchedTimelinePolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedTimelinePolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedTimelinePolicyStore {
    pub items: RefCell<BTreeMap<String, BatchedTimelinePolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedTimelinePolicyStore for MemoryBatchedTimelinePolicyStore {
    fn find(&self, code: &str) -> Option<BatchedTimelinePolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedTimelinePolicy) {
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

    fn list(&self) -> Vec<BatchedTimelinePolicy> {
        let mut out: Vec<BatchedTimelinePolicy> = Vec::new();
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

pub fn new_memory_batched_timeline_policy_store() -> Rc<dyn BatchedTimelinePolicyStore> {
    Rc::new(MemoryBatchedTimelinePolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedTimelinePolicyStore {
    pub inner: Rc<dyn BatchedTimelinePolicyStore>,
    pub cache: RefCell<cache::TtlCache<BatchedTimelinePolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedTimelinePolicyStore for CachedBatchedTimelinePolicyStore {
    fn find(&self, code: &str) -> Option<BatchedTimelinePolicy> {
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

    fn save(&self, item: &BatchedTimelinePolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedTimelinePolicy> {
        self.inner.list()
    }
}

pub fn new_cached_batched_timeline_policy_store(
    inner: Rc<dyn BatchedTimelinePolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedTimelinePolicyStore> {
    Rc::new(CachedBatchedTimelinePolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedTimelinePolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched timeline policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelinePolicyQuery {
    pub title: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedTimelinePolicyQuery {
    pub fn matches(&self, item: &BatchedTimelinePolicy) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[BatchedTimelinePolicy]) -> Vec<BatchedTimelinePolicy> {
        let matched = collections::filter(items, |candidate: &BatchedTimelinePolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_timeline_policies(
    store: Rc<dyn BatchedTimelinePolicyStore>,
    query: &BatchedTimelinePolicyQuery,
) -> Vec<BatchedTimelinePolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedTimelinePolicyStore> = new_memory_batched_timeline_policy_store();
        let item = new_batched_timeline_policy("code-252", "title-713");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
