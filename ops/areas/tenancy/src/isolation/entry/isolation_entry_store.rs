use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps isolation entries by code.
pub trait IsolationEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IsolationEntry>;
    fn save(&self, item: &IsolationEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IsolationEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIsolationEntryStore {
    pub items: RefCell<BTreeMap<String, IsolationEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl IsolationEntryStore for MemoryIsolationEntryStore {
    fn find(&self, code: &str) -> Option<IsolationEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IsolationEntry) {
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

    fn list(&self) -> Vec<IsolationEntry> {
        let mut out: Vec<IsolationEntry> = Vec::new();
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

pub fn new_memory_isolation_entry_store() -> Rc<dyn IsolationEntryStore> {
    Rc::new(MemoryIsolationEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIsolationEntryStore {
    pub inner: Rc<dyn IsolationEntryStore>,
    pub cache: RefCell<cache::TtlCache<IsolationEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IsolationEntryStore for CachedIsolationEntryStore {
    fn find(&self, code: &str) -> Option<IsolationEntry> {
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

    fn save(&self, item: &IsolationEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IsolationEntry> {
        self.inner.list()
    }
}

pub fn new_cached_isolation_entry_store(
    inner: Rc<dyn IsolationEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IsolationEntryStore> {
    Rc::new(CachedIsolationEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IsolationEntry>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects isolation entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEntryQuery {
    pub source: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IsolationEntryQuery {
    pub fn matches(&self, item: &IsolationEntry) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[IsolationEntry]) -> Vec<IsolationEntry> {
        let matched = collections::filter(items, |candidate: &IsolationEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_isolation_entries(store: Rc<dyn IsolationEntryStore>, query: &IsolationEntryQuery) -> Vec<IsolationEntry> {
    query.apply(&store.list())
}
