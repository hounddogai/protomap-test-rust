use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive runbook entries by code.
pub trait AdaptiveRunbookEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveRunbookEntry>;
    fn save(&self, item: &AdaptiveRunbookEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveRunbookEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveRunbookEntryStore {
    pub items: RefCell<BTreeMap<String, AdaptiveRunbookEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveRunbookEntryStore for MemoryAdaptiveRunbookEntryStore {
    fn find(&self, code: &str) -> Option<AdaptiveRunbookEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveRunbookEntry) {
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

    fn list(&self) -> Vec<AdaptiveRunbookEntry> {
        let mut out: Vec<AdaptiveRunbookEntry> = Vec::new();
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

pub fn new_memory_adaptive_runbook_entry_store() -> Rc<dyn AdaptiveRunbookEntryStore> {
    Rc::new(MemoryAdaptiveRunbookEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveRunbookEntryStore {
    pub inner: Rc<dyn AdaptiveRunbookEntryStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveRunbookEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveRunbookEntryStore for CachedAdaptiveRunbookEntryStore {
    fn find(&self, code: &str) -> Option<AdaptiveRunbookEntry> {
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

    fn save(&self, item: &AdaptiveRunbookEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveRunbookEntry> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_runbook_entry_store(
    inner: Rc<dyn AdaptiveRunbookEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveRunbookEntryStore> {
    Rc::new(CachedAdaptiveRunbookEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveRunbookEntry>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive runbook entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookEntryQuery {
    pub namespace: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveRunbookEntryQuery {
    pub fn matches(&self, item: &AdaptiveRunbookEntry) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveRunbookEntry]) -> Vec<AdaptiveRunbookEntry> {
        let matched = collections::filter(items, |candidate: &AdaptiveRunbookEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_runbook_entries(
    store: Rc<dyn AdaptiveRunbookEntryStore>,
    query: &AdaptiveRunbookEntryQuery,
) -> Vec<AdaptiveRunbookEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveRunbookEntryStore> = new_memory_adaptive_runbook_entry_store();
        let item = new_adaptive_runbook_entry("code-140", "title-576");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
