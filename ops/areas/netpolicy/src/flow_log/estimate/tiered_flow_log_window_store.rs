use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered flow log windows by code.
pub trait TieredFlowLogWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredFlowLogWindow>;
    fn save(&self, item: &TieredFlowLogWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredFlowLogWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredFlowLogWindowStore {
    pub items: RefCell<BTreeMap<String, TieredFlowLogWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredFlowLogWindowStore for MemoryTieredFlowLogWindowStore {
    fn find(&self, code: &str) -> Option<TieredFlowLogWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredFlowLogWindow) {
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

    fn list(&self) -> Vec<TieredFlowLogWindow> {
        let mut out: Vec<TieredFlowLogWindow> = Vec::new();
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

pub fn new_memory_tiered_flow_log_window_store() -> Rc<dyn TieredFlowLogWindowStore> {
    Rc::new(MemoryTieredFlowLogWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredFlowLogWindowStore {
    pub inner: Rc<dyn TieredFlowLogWindowStore>,
    pub cache: RefCell<cache::TtlCache<TieredFlowLogWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredFlowLogWindowStore for CachedTieredFlowLogWindowStore {
    fn find(&self, code: &str) -> Option<TieredFlowLogWindow> {
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

    fn save(&self, item: &TieredFlowLogWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredFlowLogWindow> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_flow_log_window_store(
    inner: Rc<dyn TieredFlowLogWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredFlowLogWindowStore> {
    Rc::new(CachedTieredFlowLogWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredFlowLogWindow>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered flow log windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogWindowQuery {
    pub category: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredFlowLogWindowQuery {
    pub fn matches(&self, item: &TieredFlowLogWindow) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredFlowLogWindow]) -> Vec<TieredFlowLogWindow> {
        let matched = collections::filter(items, |candidate: &TieredFlowLogWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_flow_log_windows(
    store: Rc<dyn TieredFlowLogWindowStore>,
    query: &TieredFlowLogWindowQuery,
) -> Vec<TieredFlowLogWindow> {
    query.apply(&store.list())
}
