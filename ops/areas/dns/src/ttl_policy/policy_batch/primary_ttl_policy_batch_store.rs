use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary ttl policy batches by code.
pub trait PrimaryTtlPolicyBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryTtlPolicyBatch>;
    fn save(&self, item: &PrimaryTtlPolicyBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryTtlPolicyBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryTtlPolicyBatchStore {
    pub items: RefCell<BTreeMap<String, PrimaryTtlPolicyBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryTtlPolicyBatchStore for MemoryPrimaryTtlPolicyBatchStore {
    fn find(&self, code: &str) -> Option<PrimaryTtlPolicyBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryTtlPolicyBatch) {
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

    fn list(&self) -> Vec<PrimaryTtlPolicyBatch> {
        let mut out: Vec<PrimaryTtlPolicyBatch> = Vec::new();
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

pub fn new_memory_primary_ttl_policy_batch_store() -> Rc<dyn PrimaryTtlPolicyBatchStore> {
    Rc::new(MemoryPrimaryTtlPolicyBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryTtlPolicyBatchStore {
    pub inner: Rc<dyn PrimaryTtlPolicyBatchStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryTtlPolicyBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryTtlPolicyBatchStore for CachedPrimaryTtlPolicyBatchStore {
    fn find(&self, code: &str) -> Option<PrimaryTtlPolicyBatch> {
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

    fn save(&self, item: &PrimaryTtlPolicyBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryTtlPolicyBatch> {
        self.inner.list()
    }
}

pub fn new_cached_primary_ttl_policy_batch_store(
    inner: Rc<dyn PrimaryTtlPolicyBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryTtlPolicyBatchStore> {
    Rc::new(CachedPrimaryTtlPolicyBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryTtlPolicyBatch>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary ttl policy batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyBatchQuery {
    pub cluster: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryTtlPolicyBatchQuery {
    pub fn matches(&self, item: &PrimaryTtlPolicyBatch) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryTtlPolicyBatch]) -> Vec<PrimaryTtlPolicyBatch> {
        let matched = collections::filter(items, |candidate: &PrimaryTtlPolicyBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_ttl_policy_batches(
    store: Rc<dyn PrimaryTtlPolicyBatchStore>,
    query: &PrimaryTtlPolicyBatchQuery,
) -> Vec<PrimaryTtlPolicyBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryTtlPolicyBatchStore> = new_memory_primary_ttl_policy_batch_store();
        let item = new_primary_ttl_policy_batch("code-536", "title-276");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
