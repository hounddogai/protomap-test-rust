use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched policy tiers by code.
pub trait BatchedPolicyTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedPolicyTier>;
    fn save(&self, item: &BatchedPolicyTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedPolicyTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedPolicyTierStore {
    pub items: RefCell<BTreeMap<String, BatchedPolicyTier>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedPolicyTierStore for MemoryBatchedPolicyTierStore {
    fn find(&self, code: &str) -> Option<BatchedPolicyTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedPolicyTier) {
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

    fn list(&self) -> Vec<BatchedPolicyTier> {
        let mut out: Vec<BatchedPolicyTier> = Vec::new();
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

pub fn new_memory_batched_policy_tier_store() -> Rc<dyn BatchedPolicyTierStore> {
    Rc::new(MemoryBatchedPolicyTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedPolicyTierStore {
    pub inner: Rc<dyn BatchedPolicyTierStore>,
    pub cache: RefCell<cache::TtlCache<BatchedPolicyTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedPolicyTierStore for CachedBatchedPolicyTierStore {
    fn find(&self, code: &str) -> Option<BatchedPolicyTier> {
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

    fn save(&self, item: &BatchedPolicyTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedPolicyTier> {
        self.inner.list()
    }
}

pub fn new_cached_batched_policy_tier_store(
    inner: Rc<dyn BatchedPolicyTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedPolicyTierStore> {
    Rc::new(CachedBatchedPolicyTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedPolicyTier>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched policy tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTierQuery {
    pub segment: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedPolicyTierQuery {
    pub fn matches(&self, item: &BatchedPolicyTier) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[BatchedPolicyTier]) -> Vec<BatchedPolicyTier> {
        let matched = collections::filter(items, |candidate: &BatchedPolicyTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_policy_tiers(
    store: Rc<dyn BatchedPolicyTierStore>,
    query: &BatchedPolicyTierQuery,
) -> Vec<BatchedPolicyTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedPolicyTierStore> = new_memory_batched_policy_tier_store();
        let item = new_batched_policy_tier("code-788", "name-939");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
