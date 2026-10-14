use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps runbook entries by code.
pub trait RunbookEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RunbookEntry>;
    fn save(&self, item: &RunbookEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RunbookEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRunbookEntryStore {
    pub items: RefCell<BTreeMap<String, RunbookEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl RunbookEntryStore for MemoryRunbookEntryStore {
    fn find(&self, code: &str) -> Option<RunbookEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RunbookEntry) {
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

    fn list(&self) -> Vec<RunbookEntry> {
        let mut out: Vec<RunbookEntry> = Vec::new();
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

pub fn new_memory_runbook_entry_store() -> Rc<dyn RunbookEntryStore> {
    Rc::new(MemoryRunbookEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRunbookEntryStore {
    pub inner: Rc<dyn RunbookEntryStore>,
    pub cache: RefCell<cache::TtlCache<RunbookEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RunbookEntryStore for CachedRunbookEntryStore {
    fn find(&self, code: &str) -> Option<RunbookEntry> {
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

    fn save(&self, item: &RunbookEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RunbookEntry> {
        self.inner.list()
    }
}

pub fn new_cached_runbook_entry_store(
    inner: Rc<dyn RunbookEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RunbookEntryStore> {
    Rc::new(CachedRunbookEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RunbookEntry>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects runbook entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEntryQuery {
    pub country: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RunbookEntryQuery {
    pub fn matches(&self, item: &RunbookEntry) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RunbookEntry]) -> Vec<RunbookEntry> {
        let matched = collections::filter(items, |candidate: &RunbookEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_runbook_entries(store: Rc<dyn RunbookEntryStore>, query: &RunbookEntryQuery) -> Vec<RunbookEntry> {
    query.apply(&store.list())
}
