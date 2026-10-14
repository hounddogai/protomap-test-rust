use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched policy exceptions by code.
pub trait BatchedPolicyExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedPolicyException>;
    fn save(&self, item: &BatchedPolicyException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedPolicyException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedPolicyExceptionStore {
    pub items: RefCell<BTreeMap<String, BatchedPolicyException>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedPolicyExceptionStore for MemoryBatchedPolicyExceptionStore {
    fn find(&self, code: &str) -> Option<BatchedPolicyException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedPolicyException) {
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

    fn list(&self) -> Vec<BatchedPolicyException> {
        let mut out: Vec<BatchedPolicyException> = Vec::new();
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

pub fn new_memory_batched_policy_exception_store() -> Rc<dyn BatchedPolicyExceptionStore> {
    Rc::new(MemoryBatchedPolicyExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedPolicyExceptionStore {
    pub inner: Rc<dyn BatchedPolicyExceptionStore>,
    pub cache: RefCell<cache::TtlCache<BatchedPolicyException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedPolicyExceptionStore for CachedBatchedPolicyExceptionStore {
    fn find(&self, code: &str) -> Option<BatchedPolicyException> {
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

    fn save(&self, item: &BatchedPolicyException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedPolicyException> {
        self.inner.list()
    }
}

pub fn new_cached_batched_policy_exception_store(
    inner: Rc<dyn BatchedPolicyExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedPolicyExceptionStore> {
    Rc::new(CachedBatchedPolicyExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedPolicyException>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched policy exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyExceptionQuery {
    pub label: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedPolicyExceptionQuery {
    pub fn matches(&self, item: &BatchedPolicyException) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedPolicyException]) -> Vec<BatchedPolicyException> {
        let matched = collections::filter(items, |candidate: &BatchedPolicyException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_policy_exceptions(
    store: Rc<dyn BatchedPolicyExceptionStore>,
    query: &BatchedPolicyExceptionQuery,
) -> Vec<BatchedPolicyException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedPolicyExceptionStore> = new_memory_batched_policy_exception_store();
        let item = new_batched_policy_exception("code-329", "label-86");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
