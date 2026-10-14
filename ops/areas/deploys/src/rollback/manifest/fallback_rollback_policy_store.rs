use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback rollback policies by code.
pub trait FallbackRollbackPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackRollbackPolicy>;
    fn save(&self, item: &FallbackRollbackPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackRollbackPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackRollbackPolicyStore {
    pub items: RefCell<BTreeMap<String, FallbackRollbackPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackRollbackPolicyStore for MemoryFallbackRollbackPolicyStore {
    fn find(&self, code: &str) -> Option<FallbackRollbackPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackRollbackPolicy) {
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

    fn list(&self) -> Vec<FallbackRollbackPolicy> {
        let mut out: Vec<FallbackRollbackPolicy> = Vec::new();
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

pub fn new_memory_fallback_rollback_policy_store() -> Rc<dyn FallbackRollbackPolicyStore> {
    Rc::new(MemoryFallbackRollbackPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackRollbackPolicyStore {
    pub inner: Rc<dyn FallbackRollbackPolicyStore>,
    pub cache: RefCell<cache::TtlCache<FallbackRollbackPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackRollbackPolicyStore for CachedFallbackRollbackPolicyStore {
    fn find(&self, code: &str) -> Option<FallbackRollbackPolicy> {
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

    fn save(&self, item: &FallbackRollbackPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackRollbackPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_rollback_policy_store(
    inner: Rc<dyn FallbackRollbackPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackRollbackPolicyStore> {
    Rc::new(CachedFallbackRollbackPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackRollbackPolicy>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback rollback policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackPolicyQuery {
    pub region: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackRollbackPolicyQuery {
    pub fn matches(&self, item: &FallbackRollbackPolicy) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackRollbackPolicy]) -> Vec<FallbackRollbackPolicy> {
        let matched = collections::filter(items, |candidate: &FallbackRollbackPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_rollback_policies(
    store: Rc<dyn FallbackRollbackPolicyStore>,
    query: &FallbackRollbackPolicyQuery,
) -> Vec<FallbackRollbackPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackRollbackPolicyStore> = new_memory_fallback_rollback_policy_store();
        let item = new_fallback_rollback_policy("code-3", "title-281");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
