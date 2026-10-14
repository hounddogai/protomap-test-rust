use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict release policies by code.
pub trait StrictReleasePolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictReleasePolicy>;
    fn save(&self, item: &StrictReleasePolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictReleasePolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictReleasePolicyStore {
    pub items: RefCell<BTreeMap<String, StrictReleasePolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictReleasePolicyStore for MemoryStrictReleasePolicyStore {
    fn find(&self, code: &str) -> Option<StrictReleasePolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictReleasePolicy) {
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

    fn list(&self) -> Vec<StrictReleasePolicy> {
        let mut out: Vec<StrictReleasePolicy> = Vec::new();
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

pub fn new_memory_strict_release_policy_store() -> Rc<dyn StrictReleasePolicyStore> {
    Rc::new(MemoryStrictReleasePolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictReleasePolicyStore {
    pub inner: Rc<dyn StrictReleasePolicyStore>,
    pub cache: RefCell<cache::TtlCache<StrictReleasePolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictReleasePolicyStore for CachedStrictReleasePolicyStore {
    fn find(&self, code: &str) -> Option<StrictReleasePolicy> {
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

    fn save(&self, item: &StrictReleasePolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictReleasePolicy> {
        self.inner.list()
    }
}

pub fn new_cached_strict_release_policy_store(
    inner: Rc<dyn StrictReleasePolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictReleasePolicyStore> {
    Rc::new(CachedStrictReleasePolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictReleasePolicy>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict release policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleasePolicyQuery {
    pub category: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictReleasePolicyQuery {
    pub fn matches(&self, item: &StrictReleasePolicy) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictReleasePolicy]) -> Vec<StrictReleasePolicy> {
        let matched = collections::filter(items, |candidate: &StrictReleasePolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_release_policies(
    store: Rc<dyn StrictReleasePolicyStore>,
    query: &StrictReleasePolicyQuery,
) -> Vec<StrictReleasePolicy> {
    query.apply(&store.list())
}
