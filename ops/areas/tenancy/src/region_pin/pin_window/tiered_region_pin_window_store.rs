use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered region pin windows by code.
pub trait TieredRegionPinWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredRegionPinWindow>;
    fn save(&self, item: &TieredRegionPinWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredRegionPinWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredRegionPinWindowStore {
    pub items: RefCell<BTreeMap<String, TieredRegionPinWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredRegionPinWindowStore for MemoryTieredRegionPinWindowStore {
    fn find(&self, code: &str) -> Option<TieredRegionPinWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredRegionPinWindow) {
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

    fn list(&self) -> Vec<TieredRegionPinWindow> {
        let mut out: Vec<TieredRegionPinWindow> = Vec::new();
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

pub fn new_memory_tiered_region_pin_window_store() -> Rc<dyn TieredRegionPinWindowStore> {
    Rc::new(MemoryTieredRegionPinWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredRegionPinWindowStore {
    pub inner: Rc<dyn TieredRegionPinWindowStore>,
    pub cache: RefCell<cache::TtlCache<TieredRegionPinWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredRegionPinWindowStore for CachedTieredRegionPinWindowStore {
    fn find(&self, code: &str) -> Option<TieredRegionPinWindow> {
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

    fn save(&self, item: &TieredRegionPinWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredRegionPinWindow> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_region_pin_window_store(
    inner: Rc<dyn TieredRegionPinWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredRegionPinWindowStore> {
    Rc::new(CachedTieredRegionPinWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredRegionPinWindow>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered region pin windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinWindowQuery {
    pub title: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredRegionPinWindowQuery {
    pub fn matches(&self, item: &TieredRegionPinWindow) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[TieredRegionPinWindow]) -> Vec<TieredRegionPinWindow> {
        let matched = collections::filter(items, |candidate: &TieredRegionPinWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_region_pin_windows(
    store: Rc<dyn TieredRegionPinWindowStore>,
    query: &TieredRegionPinWindowQuery,
) -> Vec<TieredRegionPinWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredRegionPinWindowStore> = new_memory_tiered_region_pin_window_store();
        let item = new_tiered_region_pin_window("code-787", "title-673");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
