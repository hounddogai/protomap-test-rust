use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps timeline rollups by code.
pub trait TimelineRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TimelineRollup>;
    fn save(&self, item: &TimelineRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TimelineRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTimelineRollupStore {
    pub items: RefCell<BTreeMap<String, TimelineRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl TimelineRollupStore for MemoryTimelineRollupStore {
    fn find(&self, code: &str) -> Option<TimelineRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TimelineRollup) {
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

    fn list(&self) -> Vec<TimelineRollup> {
        let mut out: Vec<TimelineRollup> = Vec::new();
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

pub fn new_memory_timeline_rollup_store() -> Rc<dyn TimelineRollupStore> {
    Rc::new(MemoryTimelineRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTimelineRollupStore {
    pub inner: Rc<dyn TimelineRollupStore>,
    pub cache: RefCell<cache::TtlCache<TimelineRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TimelineRollupStore for CachedTimelineRollupStore {
    fn find(&self, code: &str) -> Option<TimelineRollup> {
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

    fn save(&self, item: &TimelineRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TimelineRollup> {
        self.inner.list()
    }
}

pub fn new_cached_timeline_rollup_store(
    inner: Rc<dyn TimelineRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TimelineRollupStore> {
    Rc::new(CachedTimelineRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TimelineRollup>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects timeline rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRollupQuery {
    pub label: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TimelineRollupQuery {
    pub fn matches(&self, item: &TimelineRollup) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TimelineRollup]) -> Vec<TimelineRollup> {
        let matched = collections::filter(items, |candidate: &TimelineRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_timeline_rollups(store: Rc<dyn TimelineRollupStore>, query: &TimelineRollupQuery) -> Vec<TimelineRollup> {
    query.apply(&store.list())
}
