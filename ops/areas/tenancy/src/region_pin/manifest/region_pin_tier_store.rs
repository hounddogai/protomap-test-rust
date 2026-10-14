use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps region pin tiers by code.
pub trait RegionPinTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionPinTier>;
    fn save(&self, item: &RegionPinTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionPinTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionPinTierStore {
    pub items: RefCell<BTreeMap<String, RegionPinTier>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionPinTierStore for MemoryRegionPinTierStore {
    fn find(&self, code: &str) -> Option<RegionPinTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionPinTier) {
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

    fn list(&self) -> Vec<RegionPinTier> {
        let mut out: Vec<RegionPinTier> = Vec::new();
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

pub fn new_memory_region_pin_tier_store() -> Rc<dyn RegionPinTierStore> {
    Rc::new(MemoryRegionPinTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionPinTierStore {
    pub inner: Rc<dyn RegionPinTierStore>,
    pub cache: RefCell<cache::TtlCache<RegionPinTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionPinTierStore for CachedRegionPinTierStore {
    fn find(&self, code: &str) -> Option<RegionPinTier> {
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

    fn save(&self, item: &RegionPinTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionPinTier> {
        self.inner.list()
    }
}

pub fn new_cached_region_pin_tier_store(
    inner: Rc<dyn RegionPinTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionPinTierStore> {
    Rc::new(CachedRegionPinTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionPinTier>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects region pin tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTierQuery {
    pub country: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionPinTierQuery {
    pub fn matches(&self, item: &RegionPinTier) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionPinTier]) -> Vec<RegionPinTier> {
        let matched = collections::filter(items, |candidate: &RegionPinTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_region_pin_tiers(store: Rc<dyn RegionPinTierStore>, query: &RegionPinTierQuery) -> Vec<RegionPinTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionPinTierStore> = new_memory_region_pin_tier_store();
        let item = new_region_pin_tier("code-517", "label-685");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
