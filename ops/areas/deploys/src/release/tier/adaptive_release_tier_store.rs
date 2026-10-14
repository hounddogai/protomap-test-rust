use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive release tiers by code.
pub trait AdaptiveReleaseTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveReleaseTier>;
    fn save(&self, item: &AdaptiveReleaseTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveReleaseTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveReleaseTierStore {
    pub items: RefCell<BTreeMap<String, AdaptiveReleaseTier>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveReleaseTierStore for MemoryAdaptiveReleaseTierStore {
    fn find(&self, code: &str) -> Option<AdaptiveReleaseTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveReleaseTier) {
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

    fn list(&self) -> Vec<AdaptiveReleaseTier> {
        let mut out: Vec<AdaptiveReleaseTier> = Vec::new();
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

pub fn new_memory_adaptive_release_tier_store() -> Rc<dyn AdaptiveReleaseTierStore> {
    Rc::new(MemoryAdaptiveReleaseTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveReleaseTierStore {
    pub inner: Rc<dyn AdaptiveReleaseTierStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveReleaseTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveReleaseTierStore for CachedAdaptiveReleaseTierStore {
    fn find(&self, code: &str) -> Option<AdaptiveReleaseTier> {
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

    fn save(&self, item: &AdaptiveReleaseTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveReleaseTier> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_release_tier_store(
    inner: Rc<dyn AdaptiveReleaseTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveReleaseTierStore> {
    Rc::new(CachedAdaptiveReleaseTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveReleaseTier>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive release tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseTierQuery {
    pub namespace: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveReleaseTierQuery {
    pub fn matches(&self, item: &AdaptiveReleaseTier) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveReleaseTier]) -> Vec<AdaptiveReleaseTier> {
        let matched = collections::filter(items, |candidate: &AdaptiveReleaseTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_release_tiers(
    store: Rc<dyn AdaptiveReleaseTierStore>,
    query: &AdaptiveReleaseTierQuery,
) -> Vec<AdaptiveReleaseTier> {
    query.apply(&store.list())
}
