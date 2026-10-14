use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity entries by code.
pub trait IdentityEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityEntry>;
    fn save(&self, item: &IdentityEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityEntryStore {
    pub items: RefCell<BTreeMap<String, IdentityEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityEntryStore for MemoryIdentityEntryStore {
    fn find(&self, code: &str) -> Option<IdentityEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityEntry) {
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

    fn list(&self) -> Vec<IdentityEntry> {
        let mut out: Vec<IdentityEntry> = Vec::new();
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

pub fn new_memory_identity_entry_store() -> Rc<dyn IdentityEntryStore> {
    Rc::new(MemoryIdentityEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityEntryStore {
    pub inner: Rc<dyn IdentityEntryStore>,
    pub cache: RefCell<cache::TtlCache<IdentityEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityEntryStore for CachedIdentityEntryStore {
    fn find(&self, code: &str) -> Option<IdentityEntry> {
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

    fn save(&self, item: &IdentityEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityEntry> {
        self.inner.list()
    }
}

pub fn new_cached_identity_entry_store(
    inner: Rc<dyn IdentityEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityEntryStore> {
    Rc::new(CachedIdentityEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityEntry>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntryQuery {
    pub channel: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityEntryQuery {
    pub fn matches(&self, item: &IdentityEntry) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IdentityEntry]) -> Vec<IdentityEntry> {
        let matched = collections::filter(items, |candidate: &IdentityEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_entries(store: Rc<dyn IdentityEntryStore>, query: &IdentityEntryQuery) -> Vec<IdentityEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IdentityEntryStore> = new_memory_identity_entry_store();
        let item = new_identity_entry("code-318", "title-962");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
