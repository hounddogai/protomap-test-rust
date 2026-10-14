use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps policy batches by code.
pub trait PolicyBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PolicyBatch>;
    fn save(&self, item: &PolicyBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PolicyBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPolicyBatchStore {
    pub items: RefCell<BTreeMap<String, PolicyBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl PolicyBatchStore for MemoryPolicyBatchStore {
    fn find(&self, code: &str) -> Option<PolicyBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PolicyBatch) {
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

    fn list(&self) -> Vec<PolicyBatch> {
        let mut out: Vec<PolicyBatch> = Vec::new();
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

pub fn new_memory_policy_batch_store() -> Rc<dyn PolicyBatchStore> {
    Rc::new(MemoryPolicyBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPolicyBatchStore {
    pub inner: Rc<dyn PolicyBatchStore>,
    pub cache: RefCell<cache::TtlCache<PolicyBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PolicyBatchStore for CachedPolicyBatchStore {
    fn find(&self, code: &str) -> Option<PolicyBatch> {
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

    fn save(&self, item: &PolicyBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PolicyBatch> {
        self.inner.list()
    }
}

pub fn new_cached_policy_batch_store(
    inner: Rc<dyn PolicyBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PolicyBatchStore> {
    Rc::new(CachedPolicyBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PolicyBatch>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects policy batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyBatchQuery {
    pub category: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PolicyBatchQuery {
    pub fn matches(&self, item: &PolicyBatch) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[PolicyBatch]) -> Vec<PolicyBatch> {
        let matched = collections::filter(items, |candidate: &PolicyBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_policy_batches(store: Rc<dyn PolicyBatchStore>, query: &PolicyBatchQuery) -> Vec<PolicyBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PolicyBatchStore> = new_memory_policy_batch_store();
        let item = new_policy_batch("code-687", "name-34");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
