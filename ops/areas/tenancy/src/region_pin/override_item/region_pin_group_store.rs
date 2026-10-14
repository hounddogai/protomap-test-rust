use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps region pin groups by code.
pub trait RegionPinGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionPinGroup>;
    fn save(&self, item: &RegionPinGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionPinGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionPinGroupStore {
    pub items: RefCell<BTreeMap<String, RegionPinGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionPinGroupStore for MemoryRegionPinGroupStore {
    fn find(&self, code: &str) -> Option<RegionPinGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionPinGroup) {
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

    fn list(&self) -> Vec<RegionPinGroup> {
        let mut out: Vec<RegionPinGroup> = Vec::new();
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

pub fn new_memory_region_pin_group_store() -> Rc<dyn RegionPinGroupStore> {
    Rc::new(MemoryRegionPinGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionPinGroupStore {
    pub inner: Rc<dyn RegionPinGroupStore>,
    pub cache: RefCell<cache::TtlCache<RegionPinGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionPinGroupStore for CachedRegionPinGroupStore {
    fn find(&self, code: &str) -> Option<RegionPinGroup> {
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

    fn save(&self, item: &RegionPinGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionPinGroup> {
        self.inner.list()
    }
}

pub fn new_cached_region_pin_group_store(
    inner: Rc<dyn RegionPinGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionPinGroupStore> {
    Rc::new(CachedRegionPinGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionPinGroup>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects region pin groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinGroupQuery {
    pub title: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionPinGroupQuery {
    pub fn matches(&self, item: &RegionPinGroup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionPinGroup]) -> Vec<RegionPinGroup> {
        let matched = collections::filter(items, |candidate: &RegionPinGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_region_pin_groups(store: Rc<dyn RegionPinGroupStore>, query: &RegionPinGroupQuery) -> Vec<RegionPinGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionPinGroupStore> = new_memory_region_pin_group_store();
        let item = new_region_pin_group("code-288", "title-121");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
