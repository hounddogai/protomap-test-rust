use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps probe windows by code.
pub trait ProbeWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ProbeWindow>;
    fn save(&self, item: &ProbeWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ProbeWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryProbeWindowStore {
    pub items: RefCell<BTreeMap<String, ProbeWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl ProbeWindowStore for MemoryProbeWindowStore {
    fn find(&self, code: &str) -> Option<ProbeWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ProbeWindow) {
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

    fn list(&self) -> Vec<ProbeWindow> {
        let mut out: Vec<ProbeWindow> = Vec::new();
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

pub fn new_memory_probe_window_store() -> Rc<dyn ProbeWindowStore> {
    Rc::new(MemoryProbeWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedProbeWindowStore {
    pub inner: Rc<dyn ProbeWindowStore>,
    pub cache: RefCell<cache::TtlCache<ProbeWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ProbeWindowStore for CachedProbeWindowStore {
    fn find(&self, code: &str) -> Option<ProbeWindow> {
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

    fn save(&self, item: &ProbeWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ProbeWindow> {
        self.inner.list()
    }
}

pub fn new_cached_probe_window_store(
    inner: Rc<dyn ProbeWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ProbeWindowStore> {
    Rc::new(CachedProbeWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ProbeWindow>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects probe windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeWindowQuery {
    pub title: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ProbeWindowQuery {
    pub fn matches(&self, item: &ProbeWindow) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ProbeWindow]) -> Vec<ProbeWindow> {
        let matched = collections::filter(items, |candidate: &ProbeWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_probe_windows(store: Rc<dyn ProbeWindowStore>, query: &ProbeWindowQuery) -> Vec<ProbeWindow> {
    query.apply(&store.list())
}
