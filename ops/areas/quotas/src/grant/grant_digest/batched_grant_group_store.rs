use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched grant groups by code.
pub trait BatchedGrantGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedGrantGroup>;
    fn save(&self, item: &BatchedGrantGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedGrantGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedGrantGroupStore {
    pub items: RefCell<BTreeMap<String, BatchedGrantGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedGrantGroupStore for MemoryBatchedGrantGroupStore {
    fn find(&self, code: &str) -> Option<BatchedGrantGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedGrantGroup) {
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

    fn list(&self) -> Vec<BatchedGrantGroup> {
        let mut out: Vec<BatchedGrantGroup> = Vec::new();
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

pub fn new_memory_batched_grant_group_store() -> Rc<dyn BatchedGrantGroupStore> {
    Rc::new(MemoryBatchedGrantGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedGrantGroupStore {
    pub inner: Rc<dyn BatchedGrantGroupStore>,
    pub cache: RefCell<cache::TtlCache<BatchedGrantGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedGrantGroupStore for CachedBatchedGrantGroupStore {
    fn find(&self, code: &str) -> Option<BatchedGrantGroup> {
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

    fn save(&self, item: &BatchedGrantGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedGrantGroup> {
        self.inner.list()
    }
}

pub fn new_cached_batched_grant_group_store(
    inner: Rc<dyn BatchedGrantGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedGrantGroupStore> {
    Rc::new(CachedBatchedGrantGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedGrantGroup>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched grant groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantGroupQuery {
    pub label: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedGrantGroupQuery {
    pub fn matches(&self, item: &BatchedGrantGroup) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[BatchedGrantGroup]) -> Vec<BatchedGrantGroup> {
        let matched = collections::filter(items, |candidate: &BatchedGrantGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_grant_groups(
    store: Rc<dyn BatchedGrantGroupStore>,
    query: &BatchedGrantGroupQuery,
) -> Vec<BatchedGrantGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedGrantGroupStore> = new_memory_batched_grant_group_store();
        let item = new_batched_grant_group("code-224", "label-536");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
