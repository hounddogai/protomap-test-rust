use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps region pin baselines by code.
pub trait RegionPinBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionPinBaseline>;
    fn save(&self, item: &RegionPinBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionPinBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionPinBaselineStore {
    pub items: RefCell<BTreeMap<String, RegionPinBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionPinBaselineStore for MemoryRegionPinBaselineStore {
    fn find(&self, code: &str) -> Option<RegionPinBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionPinBaseline) {
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

    fn list(&self) -> Vec<RegionPinBaseline> {
        let mut out: Vec<RegionPinBaseline> = Vec::new();
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

pub fn new_memory_region_pin_baseline_store() -> Rc<dyn RegionPinBaselineStore> {
    Rc::new(MemoryRegionPinBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionPinBaselineStore {
    pub inner: Rc<dyn RegionPinBaselineStore>,
    pub cache: RefCell<cache::TtlCache<RegionPinBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionPinBaselineStore for CachedRegionPinBaselineStore {
    fn find(&self, code: &str) -> Option<RegionPinBaseline> {
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

    fn save(&self, item: &RegionPinBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionPinBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_region_pin_baseline_store(
    inner: Rc<dyn RegionPinBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionPinBaselineStore> {
    Rc::new(CachedRegionPinBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionPinBaseline>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects region pin baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBaselineQuery {
    pub label: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionPinBaselineQuery {
    pub fn matches(&self, item: &RegionPinBaseline) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[RegionPinBaseline]) -> Vec<RegionPinBaseline> {
        let matched = collections::filter(items, |candidate: &RegionPinBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_region_pin_baselines(
    store: Rc<dyn RegionPinBaselineStore>,
    query: &RegionPinBaselineQuery,
) -> Vec<RegionPinBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionPinBaselineStore> = new_memory_region_pin_baseline_store();
        let item = new_region_pin_baseline("code-333", "label-559");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
