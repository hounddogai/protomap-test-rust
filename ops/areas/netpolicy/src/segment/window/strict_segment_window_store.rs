use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict segment windows by code.
pub trait StrictSegmentWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictSegmentWindow>;
    fn save(&self, item: &StrictSegmentWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictSegmentWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictSegmentWindowStore {
    pub items: RefCell<BTreeMap<String, StrictSegmentWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictSegmentWindowStore for MemoryStrictSegmentWindowStore {
    fn find(&self, code: &str) -> Option<StrictSegmentWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictSegmentWindow) {
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

    fn list(&self) -> Vec<StrictSegmentWindow> {
        let mut out: Vec<StrictSegmentWindow> = Vec::new();
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

pub fn new_memory_strict_segment_window_store() -> Rc<dyn StrictSegmentWindowStore> {
    Rc::new(MemoryStrictSegmentWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictSegmentWindowStore {
    pub inner: Rc<dyn StrictSegmentWindowStore>,
    pub cache: RefCell<cache::TtlCache<StrictSegmentWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictSegmentWindowStore for CachedStrictSegmentWindowStore {
    fn find(&self, code: &str) -> Option<StrictSegmentWindow> {
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

    fn save(&self, item: &StrictSegmentWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictSegmentWindow> {
        self.inner.list()
    }
}

pub fn new_cached_strict_segment_window_store(
    inner: Rc<dyn StrictSegmentWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictSegmentWindowStore> {
    Rc::new(CachedStrictSegmentWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictSegmentWindow>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict segment windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentWindowQuery {
    pub name: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictSegmentWindowQuery {
    pub fn matches(&self, item: &StrictSegmentWindow) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictSegmentWindow]) -> Vec<StrictSegmentWindow> {
        let matched = collections::filter(items, |candidate: &StrictSegmentWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_segment_windows(
    store: Rc<dyn StrictSegmentWindowStore>,
    query: &StrictSegmentWindowQuery,
) -> Vec<StrictSegmentWindow> {
    query.apply(&store.list())
}
