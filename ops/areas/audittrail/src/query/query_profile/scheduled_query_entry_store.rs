use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled query entries by code.
pub trait ScheduledQueryEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledQueryEntry>;
    fn save(&self, item: &ScheduledQueryEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledQueryEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledQueryEntryStore {
    pub items: RefCell<BTreeMap<String, ScheduledQueryEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledQueryEntryStore for MemoryScheduledQueryEntryStore {
    fn find(&self, code: &str) -> Option<ScheduledQueryEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledQueryEntry) {
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

    fn list(&self) -> Vec<ScheduledQueryEntry> {
        let mut out: Vec<ScheduledQueryEntry> = Vec::new();
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

pub fn new_memory_scheduled_query_entry_store() -> Rc<dyn ScheduledQueryEntryStore> {
    Rc::new(MemoryScheduledQueryEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledQueryEntryStore {
    pub inner: Rc<dyn ScheduledQueryEntryStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledQueryEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledQueryEntryStore for CachedScheduledQueryEntryStore {
    fn find(&self, code: &str) -> Option<ScheduledQueryEntry> {
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

    fn save(&self, item: &ScheduledQueryEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledQueryEntry> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_query_entry_store(
    inner: Rc<dyn ScheduledQueryEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledQueryEntryStore> {
    Rc::new(CachedScheduledQueryEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledQueryEntry>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled query entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEntryQuery {
    pub namespace: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledQueryEntryQuery {
    pub fn matches(&self, item: &ScheduledQueryEntry) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledQueryEntry]) -> Vec<ScheduledQueryEntry> {
        let matched = collections::filter(items, |candidate: &ScheduledQueryEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_query_entries(
    store: Rc<dyn ScheduledQueryEntryStore>,
    query: &ScheduledQueryEntryQuery,
) -> Vec<ScheduledQueryEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledQueryEntryStore> = new_memory_scheduled_query_entry_store();
        let item = new_scheduled_query_entry("code-392", "label-727");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
