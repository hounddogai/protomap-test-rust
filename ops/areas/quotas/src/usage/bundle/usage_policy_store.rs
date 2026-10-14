use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps usage policies by code.
pub trait UsagePolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<UsagePolicy>;
    fn save(&self, item: &UsagePolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<UsagePolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryUsagePolicyStore {
    pub items: RefCell<BTreeMap<String, UsagePolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl UsagePolicyStore for MemoryUsagePolicyStore {
    fn find(&self, code: &str) -> Option<UsagePolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &UsagePolicy) {
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

    fn list(&self) -> Vec<UsagePolicy> {
        let mut out: Vec<UsagePolicy> = Vec::new();
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

pub fn new_memory_usage_policy_store() -> Rc<dyn UsagePolicyStore> {
    Rc::new(MemoryUsagePolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedUsagePolicyStore {
    pub inner: Rc<dyn UsagePolicyStore>,
    pub cache: RefCell<cache::TtlCache<UsagePolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl UsagePolicyStore for CachedUsagePolicyStore {
    fn find(&self, code: &str) -> Option<UsagePolicy> {
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

    fn save(&self, item: &UsagePolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<UsagePolicy> {
        self.inner.list()
    }
}

pub fn new_cached_usage_policy_store(
    inner: Rc<dyn UsagePolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn UsagePolicyStore> {
    Rc::new(CachedUsagePolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<UsagePolicy>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects usage policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct UsagePolicyQuery {
    pub label: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl UsagePolicyQuery {
    pub fn matches(&self, item: &UsagePolicy) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[UsagePolicy]) -> Vec<UsagePolicy> {
        let matched = collections::filter(items, |candidate: &UsagePolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_usage_policies(store: Rc<dyn UsagePolicyStore>, query: &UsagePolicyQuery) -> Vec<UsagePolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn UsagePolicyStore> = new_memory_usage_policy_store();
        let item = new_usage_policy("code-474", "label-747");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
