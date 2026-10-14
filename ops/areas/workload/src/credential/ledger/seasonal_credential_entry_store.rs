use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal credential entries by code.
pub trait SeasonalCredentialEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalCredentialEntry>;
    fn save(&self, item: &SeasonalCredentialEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalCredentialEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalCredentialEntryStore {
    pub items: RefCell<BTreeMap<String, SeasonalCredentialEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalCredentialEntryStore for MemorySeasonalCredentialEntryStore {
    fn find(&self, code: &str) -> Option<SeasonalCredentialEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalCredentialEntry) {
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

    fn list(&self) -> Vec<SeasonalCredentialEntry> {
        let mut out: Vec<SeasonalCredentialEntry> = Vec::new();
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

pub fn new_memory_seasonal_credential_entry_store() -> Rc<dyn SeasonalCredentialEntryStore> {
    Rc::new(MemorySeasonalCredentialEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalCredentialEntryStore {
    pub inner: Rc<dyn SeasonalCredentialEntryStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalCredentialEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalCredentialEntryStore for CachedSeasonalCredentialEntryStore {
    fn find(&self, code: &str) -> Option<SeasonalCredentialEntry> {
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

    fn save(&self, item: &SeasonalCredentialEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalCredentialEntry> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_credential_entry_store(
    inner: Rc<dyn SeasonalCredentialEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalCredentialEntryStore> {
    Rc::new(CachedSeasonalCredentialEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalCredentialEntry>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal credential entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialEntryQuery {
    pub cluster: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalCredentialEntryQuery {
    pub fn matches(&self, item: &SeasonalCredentialEntry) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalCredentialEntry]) -> Vec<SeasonalCredentialEntry> {
        let matched = collections::filter(items, |candidate: &SeasonalCredentialEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_credential_entries(
    store: Rc<dyn SeasonalCredentialEntryStore>,
    query: &SeasonalCredentialEntryQuery,
) -> Vec<SeasonalCredentialEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalCredentialEntryStore> = new_memory_seasonal_credential_entry_store();
        let item = new_seasonal_credential_entry("code-685", "label-401");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
