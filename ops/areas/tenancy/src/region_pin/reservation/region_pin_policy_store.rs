use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps region pin policies by code.
pub trait RegionPinPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionPinPolicy>;
    fn save(&self, item: &RegionPinPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionPinPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionPinPolicyStore {
    pub items: RefCell<BTreeMap<String, RegionPinPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionPinPolicyStore for MemoryRegionPinPolicyStore {
    fn find(&self, code: &str) -> Option<RegionPinPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionPinPolicy) {
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

    fn list(&self) -> Vec<RegionPinPolicy> {
        let mut out: Vec<RegionPinPolicy> = Vec::new();
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

pub fn new_memory_region_pin_policy_store() -> Rc<dyn RegionPinPolicyStore> {
    Rc::new(MemoryRegionPinPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionPinPolicyStore {
    pub inner: Rc<dyn RegionPinPolicyStore>,
    pub cache: RefCell<cache::TtlCache<RegionPinPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionPinPolicyStore for CachedRegionPinPolicyStore {
    fn find(&self, code: &str) -> Option<RegionPinPolicy> {
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

    fn save(&self, item: &RegionPinPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionPinPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_region_pin_policy_store(
    inner: Rc<dyn RegionPinPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionPinPolicyStore> {
    Rc::new(CachedRegionPinPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionPinPolicy>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects region pin policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPolicyQuery {
    pub title: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionPinPolicyQuery {
    pub fn matches(&self, item: &RegionPinPolicy) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionPinPolicy]) -> Vec<RegionPinPolicy> {
        let matched = collections::filter(items, |candidate: &RegionPinPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_region_pin_policies(
    store: Rc<dyn RegionPinPolicyStore>,
    query: &RegionPinPolicyQuery,
) -> Vec<RegionPinPolicy> {
    query.apply(&store.list())
}
