use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive region pin overrides by code.
pub trait AdaptiveRegionPinOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinOverride>;
    fn save(&self, item: &AdaptiveRegionPinOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveRegionPinOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveRegionPinOverrideStore {
    pub items: RefCell<BTreeMap<String, AdaptiveRegionPinOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveRegionPinOverrideStore for MemoryAdaptiveRegionPinOverrideStore {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveRegionPinOverride) {
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

    fn list(&self) -> Vec<AdaptiveRegionPinOverride> {
        let mut out: Vec<AdaptiveRegionPinOverride> = Vec::new();
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

pub fn new_memory_adaptive_region_pin_override_store() -> Rc<dyn AdaptiveRegionPinOverrideStore> {
    Rc::new(MemoryAdaptiveRegionPinOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveRegionPinOverrideStore {
    pub inner: Rc<dyn AdaptiveRegionPinOverrideStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveRegionPinOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveRegionPinOverrideStore for CachedAdaptiveRegionPinOverrideStore {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinOverride> {
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

    fn save(&self, item: &AdaptiveRegionPinOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveRegionPinOverride> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_region_pin_override_store(
    inner: Rc<dyn AdaptiveRegionPinOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveRegionPinOverrideStore> {
    Rc::new(CachedAdaptiveRegionPinOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveRegionPinOverride>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive region pin overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinOverrideQuery {
    pub unit: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveRegionPinOverrideQuery {
    pub fn matches(&self, item: &AdaptiveRegionPinOverride) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveRegionPinOverride]) -> Vec<AdaptiveRegionPinOverride> {
        let matched = collections::filter(items, |candidate: &AdaptiveRegionPinOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_region_pin_overrides(
    store: Rc<dyn AdaptiveRegionPinOverrideStore>,
    query: &AdaptiveRegionPinOverrideQuery,
) -> Vec<AdaptiveRegionPinOverride> {
    query.apply(&store.list())
}
