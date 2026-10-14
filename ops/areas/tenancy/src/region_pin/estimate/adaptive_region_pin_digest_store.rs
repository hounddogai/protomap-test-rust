use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive region pin digests by code.
pub trait AdaptiveRegionPinDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinDigest>;
    fn save(&self, item: &AdaptiveRegionPinDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveRegionPinDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveRegionPinDigestStore {
    pub items: RefCell<BTreeMap<String, AdaptiveRegionPinDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveRegionPinDigestStore for MemoryAdaptiveRegionPinDigestStore {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveRegionPinDigest) {
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

    fn list(&self) -> Vec<AdaptiveRegionPinDigest> {
        let mut out: Vec<AdaptiveRegionPinDigest> = Vec::new();
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

pub fn new_memory_adaptive_region_pin_digest_store() -> Rc<dyn AdaptiveRegionPinDigestStore> {
    Rc::new(MemoryAdaptiveRegionPinDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveRegionPinDigestStore {
    pub inner: Rc<dyn AdaptiveRegionPinDigestStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveRegionPinDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveRegionPinDigestStore for CachedAdaptiveRegionPinDigestStore {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinDigest> {
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

    fn save(&self, item: &AdaptiveRegionPinDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveRegionPinDigest> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_region_pin_digest_store(
    inner: Rc<dyn AdaptiveRegionPinDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveRegionPinDigestStore> {
    Rc::new(CachedAdaptiveRegionPinDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveRegionPinDigest>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive region pin digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinDigestQuery {
    pub label: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveRegionPinDigestQuery {
    pub fn matches(&self, item: &AdaptiveRegionPinDigest) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveRegionPinDigest]) -> Vec<AdaptiveRegionPinDigest> {
        let matched = collections::filter(items, |candidate: &AdaptiveRegionPinDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_region_pin_digests(
    store: Rc<dyn AdaptiveRegionPinDigestStore>,
    query: &AdaptiveRegionPinDigestQuery,
) -> Vec<AdaptiveRegionPinDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveRegionPinDigestStore> = new_memory_adaptive_region_pin_digest_store();
        let item = new_adaptive_region_pin_digest("code-226", "label-763");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
