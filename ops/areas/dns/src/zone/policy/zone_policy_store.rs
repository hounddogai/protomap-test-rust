use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps zone policies by code.
pub trait ZonePolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ZonePolicy>;
    fn save(&self, item: &ZonePolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ZonePolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryZonePolicyStore {
    pub items: RefCell<BTreeMap<String, ZonePolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl ZonePolicyStore for MemoryZonePolicyStore {
    fn find(&self, code: &str) -> Option<ZonePolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ZonePolicy) {
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

    fn list(&self) -> Vec<ZonePolicy> {
        let mut out: Vec<ZonePolicy> = Vec::new();
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

pub fn new_memory_zone_policy_store() -> Rc<dyn ZonePolicyStore> {
    Rc::new(MemoryZonePolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedZonePolicyStore {
    pub inner: Rc<dyn ZonePolicyStore>,
    pub cache: RefCell<cache::TtlCache<ZonePolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ZonePolicyStore for CachedZonePolicyStore {
    fn find(&self, code: &str) -> Option<ZonePolicy> {
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

    fn save(&self, item: &ZonePolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ZonePolicy> {
        self.inner.list()
    }
}

pub fn new_cached_zone_policy_store(
    inner: Rc<dyn ZonePolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ZonePolicyStore> {
    Rc::new(CachedZonePolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ZonePolicy>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects zone policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ZonePolicyQuery {
    pub title: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ZonePolicyQuery {
    pub fn matches(&self, item: &ZonePolicy) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ZonePolicy]) -> Vec<ZonePolicy> {
        let matched = collections::filter(items, |candidate: &ZonePolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_zone_policies(store: Rc<dyn ZonePolicyStore>, query: &ZonePolicyQuery) -> Vec<ZonePolicy> {
    query.apply(&store.list())
}
