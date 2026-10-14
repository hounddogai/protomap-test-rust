use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query entries by code.
pub trait QueryEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryEntry>;
    fn save(&self, item: &QueryEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryEntryStore {
    pub items: RefCell<BTreeMap<String, QueryEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryEntryStore for MemoryQueryEntryStore {
    fn find(&self, code: &str) -> Option<QueryEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryEntry) {
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

    fn list(&self) -> Vec<QueryEntry> {
        let mut out: Vec<QueryEntry> = Vec::new();
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

pub fn new_memory_query_entry_store() -> Rc<dyn QueryEntryStore> {
    Rc::new(MemoryQueryEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryEntryStore {
    pub inner: Rc<dyn QueryEntryStore>,
    pub cache: RefCell<cache::TtlCache<QueryEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryEntryStore for CachedQueryEntryStore {
    fn find(&self, code: &str) -> Option<QueryEntry> {
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

    fn save(&self, item: &QueryEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryEntry> {
        self.inner.list()
    }
}

pub fn new_cached_query_entry_store(
    inner: Rc<dyn QueryEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryEntryStore> {
    Rc::new(CachedQueryEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryEntry>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryEntryQuery {
    pub unit: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryEntryQuery {
    pub fn matches(&self, item: &QueryEntry) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QueryEntry]) -> Vec<QueryEntry> {
        let matched = collections::filter(items, |candidate: &QueryEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_entries(store: Rc<dyn QueryEntryStore>, query: &QueryEntryQuery) -> Vec<QueryEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QueryEntryStore> = new_memory_query_entry_store();
        let item = new_query_entry("code-415", "name-290");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
