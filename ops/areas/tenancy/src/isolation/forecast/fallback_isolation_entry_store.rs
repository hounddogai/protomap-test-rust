use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback isolation entries by code.
pub trait FallbackIsolationEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackIsolationEntry>;
    fn save(&self, item: &FallbackIsolationEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackIsolationEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackIsolationEntryStore {
    pub items: RefCell<BTreeMap<String, FallbackIsolationEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackIsolationEntryStore for MemoryFallbackIsolationEntryStore {
    fn find(&self, code: &str) -> Option<FallbackIsolationEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackIsolationEntry) {
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

    fn list(&self) -> Vec<FallbackIsolationEntry> {
        let mut out: Vec<FallbackIsolationEntry> = Vec::new();
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

pub fn new_memory_fallback_isolation_entry_store() -> Rc<dyn FallbackIsolationEntryStore> {
    Rc::new(MemoryFallbackIsolationEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackIsolationEntryStore {
    pub inner: Rc<dyn FallbackIsolationEntryStore>,
    pub cache: RefCell<cache::TtlCache<FallbackIsolationEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackIsolationEntryStore for CachedFallbackIsolationEntryStore {
    fn find(&self, code: &str) -> Option<FallbackIsolationEntry> {
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

    fn save(&self, item: &FallbackIsolationEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackIsolationEntry> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_isolation_entry_store(
    inner: Rc<dyn FallbackIsolationEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackIsolationEntryStore> {
    Rc::new(CachedFallbackIsolationEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackIsolationEntry>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback isolation entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntryQuery {
    pub country: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackIsolationEntryQuery {
    pub fn matches(&self, item: &FallbackIsolationEntry) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackIsolationEntry]) -> Vec<FallbackIsolationEntry> {
        let matched = collections::filter(items, |candidate: &FallbackIsolationEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_isolation_entries(
    store: Rc<dyn FallbackIsolationEntryStore>,
    query: &FallbackIsolationEntryQuery,
) -> Vec<FallbackIsolationEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackIsolationEntryStore> = new_memory_fallback_isolation_entry_store();
        let item = new_fallback_isolation_entry("code-153", "name-639");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
