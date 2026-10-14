use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global release policies by code.
pub trait GlobalReleasePolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalReleasePolicy>;
    fn save(&self, item: &GlobalReleasePolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalReleasePolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalReleasePolicyStore {
    pub items: RefCell<BTreeMap<String, GlobalReleasePolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalReleasePolicyStore for MemoryGlobalReleasePolicyStore {
    fn find(&self, code: &str) -> Option<GlobalReleasePolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalReleasePolicy) {
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

    fn list(&self) -> Vec<GlobalReleasePolicy> {
        let mut out: Vec<GlobalReleasePolicy> = Vec::new();
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

pub fn new_memory_global_release_policy_store() -> Rc<dyn GlobalReleasePolicyStore> {
    Rc::new(MemoryGlobalReleasePolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalReleasePolicyStore {
    pub inner: Rc<dyn GlobalReleasePolicyStore>,
    pub cache: RefCell<cache::TtlCache<GlobalReleasePolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalReleasePolicyStore for CachedGlobalReleasePolicyStore {
    fn find(&self, code: &str) -> Option<GlobalReleasePolicy> {
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

    fn save(&self, item: &GlobalReleasePolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalReleasePolicy> {
        self.inner.list()
    }
}

pub fn new_cached_global_release_policy_store(
    inner: Rc<dyn GlobalReleasePolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalReleasePolicyStore> {
    Rc::new(CachedGlobalReleasePolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalReleasePolicy>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global release policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleasePolicyQuery {
    pub region: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalReleasePolicyQuery {
    pub fn matches(&self, item: &GlobalReleasePolicy) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalReleasePolicy]) -> Vec<GlobalReleasePolicy> {
        let matched = collections::filter(items, |candidate: &GlobalReleasePolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_release_policies(
    store: Rc<dyn GlobalReleasePolicyStore>,
    query: &GlobalReleasePolicyQuery,
) -> Vec<GlobalReleasePolicy> {
    query.apply(&store.list())
}
