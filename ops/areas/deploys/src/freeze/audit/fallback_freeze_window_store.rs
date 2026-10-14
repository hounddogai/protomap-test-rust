use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback freeze windows by code.
pub trait FallbackFreezeWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackFreezeWindow>;
    fn save(&self, item: &FallbackFreezeWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackFreezeWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackFreezeWindowStore {
    pub items: RefCell<BTreeMap<String, FallbackFreezeWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackFreezeWindowStore for MemoryFallbackFreezeWindowStore {
    fn find(&self, code: &str) -> Option<FallbackFreezeWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackFreezeWindow) {
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

    fn list(&self) -> Vec<FallbackFreezeWindow> {
        let mut out: Vec<FallbackFreezeWindow> = Vec::new();
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

pub fn new_memory_fallback_freeze_window_store() -> Rc<dyn FallbackFreezeWindowStore> {
    Rc::new(MemoryFallbackFreezeWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackFreezeWindowStore {
    pub inner: Rc<dyn FallbackFreezeWindowStore>,
    pub cache: RefCell<cache::TtlCache<FallbackFreezeWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackFreezeWindowStore for CachedFallbackFreezeWindowStore {
    fn find(&self, code: &str) -> Option<FallbackFreezeWindow> {
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

    fn save(&self, item: &FallbackFreezeWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackFreezeWindow> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_freeze_window_store(
    inner: Rc<dyn FallbackFreezeWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackFreezeWindowStore> {
    Rc::new(CachedFallbackFreezeWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackFreezeWindow>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback freeze windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeWindowQuery {
    pub namespace: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackFreezeWindowQuery {
    pub fn matches(&self, item: &FallbackFreezeWindow) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackFreezeWindow]) -> Vec<FallbackFreezeWindow> {
        let matched = collections::filter(items, |candidate: &FallbackFreezeWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_freeze_windows(
    store: Rc<dyn FallbackFreezeWindowStore>,
    query: &FallbackFreezeWindowQuery,
) -> Vec<FallbackFreezeWindow> {
    query.apply(&store.list())
}
