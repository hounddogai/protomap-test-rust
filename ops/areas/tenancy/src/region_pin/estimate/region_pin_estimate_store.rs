use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps region pin estimates by code.
pub trait RegionPinEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionPinEstimate>;
    fn save(&self, item: &RegionPinEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionPinEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionPinEstimateStore {
    pub items: RefCell<BTreeMap<String, RegionPinEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionPinEstimateStore for MemoryRegionPinEstimateStore {
    fn find(&self, code: &str) -> Option<RegionPinEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionPinEstimate) {
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

    fn list(&self) -> Vec<RegionPinEstimate> {
        let mut out: Vec<RegionPinEstimate> = Vec::new();
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

pub fn new_memory_region_pin_estimate_store() -> Rc<dyn RegionPinEstimateStore> {
    Rc::new(MemoryRegionPinEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionPinEstimateStore {
    pub inner: Rc<dyn RegionPinEstimateStore>,
    pub cache: RefCell<cache::TtlCache<RegionPinEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionPinEstimateStore for CachedRegionPinEstimateStore {
    fn find(&self, code: &str) -> Option<RegionPinEstimate> {
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

    fn save(&self, item: &RegionPinEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionPinEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_region_pin_estimate_store(
    inner: Rc<dyn RegionPinEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionPinEstimateStore> {
    Rc::new(CachedRegionPinEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionPinEstimate>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects region pin estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinEstimateQuery {
    pub label: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionPinEstimateQuery {
    pub fn matches(&self, item: &RegionPinEstimate) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionPinEstimate]) -> Vec<RegionPinEstimate> {
        let matched = collections::filter(items, |candidate: &RegionPinEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_region_pin_estimates(
    store: Rc<dyn RegionPinEstimateStore>,
    query: &RegionPinEstimateQuery,
) -> Vec<RegionPinEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionPinEstimateStore> = new_memory_region_pin_estimate_store();
        let item = new_region_pin_estimate("code-809", "label-492");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
