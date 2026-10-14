use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps limit policies by code.
pub trait LimitPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LimitPolicy>;
    fn save(&self, item: &LimitPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LimitPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLimitPolicyStore {
    pub items: RefCell<BTreeMap<String, LimitPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl LimitPolicyStore for MemoryLimitPolicyStore {
    fn find(&self, code: &str) -> Option<LimitPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LimitPolicy) {
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

    fn list(&self) -> Vec<LimitPolicy> {
        let mut out: Vec<LimitPolicy> = Vec::new();
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

pub fn new_memory_limit_policy_store() -> Rc<dyn LimitPolicyStore> {
    Rc::new(MemoryLimitPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLimitPolicyStore {
    pub inner: Rc<dyn LimitPolicyStore>,
    pub cache: RefCell<cache::TtlCache<LimitPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LimitPolicyStore for CachedLimitPolicyStore {
    fn find(&self, code: &str) -> Option<LimitPolicy> {
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

    fn save(&self, item: &LimitPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LimitPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_limit_policy_store(
    inner: Rc<dyn LimitPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LimitPolicyStore> {
    Rc::new(CachedLimitPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LimitPolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects limit policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitPolicyQuery {
    pub scope: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LimitPolicyQuery {
    pub fn matches(&self, item: &LimitPolicy) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LimitPolicy]) -> Vec<LimitPolicy> {
        let matched = collections::filter(items, |candidate: &LimitPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_limit_policies(store: Rc<dyn LimitPolicyStore>, query: &LimitPolicyQuery) -> Vec<LimitPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LimitPolicyStore> = new_memory_limit_policy_store();
        let item = new_limit_policy("code-411", "label-282");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
