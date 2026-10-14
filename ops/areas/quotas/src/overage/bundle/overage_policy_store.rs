use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps overage policies by code.
pub trait OveragePolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<OveragePolicy>;
    fn save(&self, item: &OveragePolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<OveragePolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryOveragePolicyStore {
    pub items: RefCell<BTreeMap<String, OveragePolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl OveragePolicyStore for MemoryOveragePolicyStore {
    fn find(&self, code: &str) -> Option<OveragePolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &OveragePolicy) {
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

    fn list(&self) -> Vec<OveragePolicy> {
        let mut out: Vec<OveragePolicy> = Vec::new();
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

pub fn new_memory_overage_policy_store() -> Rc<dyn OveragePolicyStore> {
    Rc::new(MemoryOveragePolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedOveragePolicyStore {
    pub inner: Rc<dyn OveragePolicyStore>,
    pub cache: RefCell<cache::TtlCache<OveragePolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl OveragePolicyStore for CachedOveragePolicyStore {
    fn find(&self, code: &str) -> Option<OveragePolicy> {
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

    fn save(&self, item: &OveragePolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<OveragePolicy> {
        self.inner.list()
    }
}

pub fn new_cached_overage_policy_store(
    inner: Rc<dyn OveragePolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn OveragePolicyStore> {
    Rc::new(CachedOveragePolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<OveragePolicy>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects overage policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct OveragePolicyQuery {
    pub country: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl OveragePolicyQuery {
    pub fn matches(&self, item: &OveragePolicy) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[OveragePolicy]) -> Vec<OveragePolicy> {
        let matched = collections::filter(items, |candidate: &OveragePolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_overage_policies(store: Rc<dyn OveragePolicyStore>, query: &OveragePolicyQuery) -> Vec<OveragePolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn OveragePolicyStore> = new_memory_overage_policy_store();
        let item = new_overage_policy("code-748", "label-955");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
