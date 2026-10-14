use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps nonce entries by code.
pub trait NonceEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<NonceEntry>;
    fn save(&self, item: &NonceEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<NonceEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNonceEntryStore {
    pub items: RefCell<BTreeMap<String, NonceEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl NonceEntryStore for MemoryNonceEntryStore {
    fn find(&self, code: &str) -> Option<NonceEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &NonceEntry) {
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

    fn list(&self) -> Vec<NonceEntry> {
        let mut out: Vec<NonceEntry> = Vec::new();
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

pub fn new_memory_nonce_entry_store() -> Rc<dyn NonceEntryStore> {
    Rc::new(MemoryNonceEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedNonceEntryStore {
    pub inner: Rc<dyn NonceEntryStore>,
    pub cache: RefCell<cache::TtlCache<NonceEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl NonceEntryStore for CachedNonceEntryStore {
    fn find(&self, code: &str) -> Option<NonceEntry> {
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

    fn save(&self, item: &NonceEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<NonceEntry> {
        self.inner.list()
    }
}

pub fn new_cached_nonce_entry_store(
    inner: Rc<dyn NonceEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn NonceEntryStore> {
    Rc::new(CachedNonceEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<NonceEntry>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects nonce entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceEntryQuery {
    pub region: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl NonceEntryQuery {
    pub fn matches(&self, item: &NonceEntry) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[NonceEntry]) -> Vec<NonceEntry> {
        let matched = collections::filter(items, |candidate: &NonceEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_nonce_entries(store: Rc<dyn NonceEntryStore>, query: &NonceEntryQuery) -> Vec<NonceEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn NonceEntryStore> = new_memory_nonce_entry_store();
        let item = new_nonce_entry("code-553", "title-689");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
