use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink entries by code.
pub trait SinkEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkEntry>;
    fn save(&self, item: &SinkEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkEntryStore {
    pub items: RefCell<BTreeMap<String, SinkEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkEntryStore for MemorySinkEntryStore {
    fn find(&self, code: &str) -> Option<SinkEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkEntry) {
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

    fn list(&self) -> Vec<SinkEntry> {
        let mut out: Vec<SinkEntry> = Vec::new();
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

pub fn new_memory_sink_entry_store() -> Rc<dyn SinkEntryStore> {
    Rc::new(MemorySinkEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkEntryStore {
    pub inner: Rc<dyn SinkEntryStore>,
    pub cache: RefCell<cache::TtlCache<SinkEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkEntryStore for CachedSinkEntryStore {
    fn find(&self, code: &str) -> Option<SinkEntry> {
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

    fn save(&self, item: &SinkEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkEntry> {
        self.inner.list()
    }
}

pub fn new_cached_sink_entry_store(
    inner: Rc<dyn SinkEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkEntryStore> {
    Rc::new(CachedSinkEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkEntry>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkEntryQuery {
    pub name: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkEntryQuery {
    pub fn matches(&self, item: &SinkEntry) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[SinkEntry]) -> Vec<SinkEntry> {
        let matched = collections::filter(items, |candidate: &SinkEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_entries(store: Rc<dyn SinkEntryStore>, query: &SinkEntryQuery) -> Vec<SinkEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SinkEntryStore> = new_memory_sink_entry_store();
        let item = new_sink_entry("code-136", "name-747");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
