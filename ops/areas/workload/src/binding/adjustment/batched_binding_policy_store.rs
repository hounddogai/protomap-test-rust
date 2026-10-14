use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched binding policies by code.
pub trait BatchedBindingPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedBindingPolicy>;
    fn save(&self, item: &BatchedBindingPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedBindingPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedBindingPolicyStore {
    pub items: RefCell<BTreeMap<String, BatchedBindingPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedBindingPolicyStore for MemoryBatchedBindingPolicyStore {
    fn find(&self, code: &str) -> Option<BatchedBindingPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedBindingPolicy) {
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

    fn list(&self) -> Vec<BatchedBindingPolicy> {
        let mut out: Vec<BatchedBindingPolicy> = Vec::new();
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

pub fn new_memory_batched_binding_policy_store() -> Rc<dyn BatchedBindingPolicyStore> {
    Rc::new(MemoryBatchedBindingPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedBindingPolicyStore {
    pub inner: Rc<dyn BatchedBindingPolicyStore>,
    pub cache: RefCell<cache::TtlCache<BatchedBindingPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedBindingPolicyStore for CachedBatchedBindingPolicyStore {
    fn find(&self, code: &str) -> Option<BatchedBindingPolicy> {
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

    fn save(&self, item: &BatchedBindingPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedBindingPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_batched_binding_policy_store(
    inner: Rc<dyn BatchedBindingPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedBindingPolicyStore> {
    Rc::new(CachedBatchedBindingPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedBindingPolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched binding policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingPolicyQuery {
    pub label: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedBindingPolicyQuery {
    pub fn matches(&self, item: &BatchedBindingPolicy) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedBindingPolicy]) -> Vec<BatchedBindingPolicy> {
        let matched = collections::filter(items, |candidate: &BatchedBindingPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_binding_policies(
    store: Rc<dyn BatchedBindingPolicyStore>,
    query: &BatchedBindingPolicyQuery,
) -> Vec<BatchedBindingPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedBindingPolicyStore> = new_memory_batched_binding_policy_store();
        let item = new_batched_binding_policy("code-868", "label-864");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
