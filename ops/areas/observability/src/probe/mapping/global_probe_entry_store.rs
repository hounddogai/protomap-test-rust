use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global probe entries by code.
pub trait GlobalProbeEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalProbeEntry>;
    fn save(&self, item: &GlobalProbeEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalProbeEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalProbeEntryStore {
    pub items: RefCell<BTreeMap<String, GlobalProbeEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalProbeEntryStore for MemoryGlobalProbeEntryStore {
    fn find(&self, code: &str) -> Option<GlobalProbeEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalProbeEntry) {
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

    fn list(&self) -> Vec<GlobalProbeEntry> {
        let mut out: Vec<GlobalProbeEntry> = Vec::new();
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

pub fn new_memory_global_probe_entry_store() -> Rc<dyn GlobalProbeEntryStore> {
    Rc::new(MemoryGlobalProbeEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalProbeEntryStore {
    pub inner: Rc<dyn GlobalProbeEntryStore>,
    pub cache: RefCell<cache::TtlCache<GlobalProbeEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalProbeEntryStore for CachedGlobalProbeEntryStore {
    fn find(&self, code: &str) -> Option<GlobalProbeEntry> {
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

    fn save(&self, item: &GlobalProbeEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalProbeEntry> {
        self.inner.list()
    }
}

pub fn new_cached_global_probe_entry_store(
    inner: Rc<dyn GlobalProbeEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalProbeEntryStore> {
    Rc::new(CachedGlobalProbeEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalProbeEntry>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global probe entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeEntryQuery {
    pub segment: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalProbeEntryQuery {
    pub fn matches(&self, item: &GlobalProbeEntry) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[GlobalProbeEntry]) -> Vec<GlobalProbeEntry> {
        let matched = collections::filter(items, |candidate: &GlobalProbeEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_probe_entries(
    store: Rc<dyn GlobalProbeEntryStore>,
    query: &GlobalProbeEntryQuery,
) -> Vec<GlobalProbeEntry> {
    query.apply(&store.list())
}
