use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered retention entries by code.
pub trait TieredRetentionEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredRetentionEntry>;
    fn save(&self, item: &TieredRetentionEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredRetentionEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredRetentionEntryStore {
    pub items: RefCell<BTreeMap<String, TieredRetentionEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredRetentionEntryStore for MemoryTieredRetentionEntryStore {
    fn find(&self, code: &str) -> Option<TieredRetentionEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredRetentionEntry) {
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

    fn list(&self) -> Vec<TieredRetentionEntry> {
        let mut out: Vec<TieredRetentionEntry> = Vec::new();
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

pub fn new_memory_tiered_retention_entry_store() -> Rc<dyn TieredRetentionEntryStore> {
    Rc::new(MemoryTieredRetentionEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredRetentionEntryStore {
    pub inner: Rc<dyn TieredRetentionEntryStore>,
    pub cache: RefCell<cache::TtlCache<TieredRetentionEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredRetentionEntryStore for CachedTieredRetentionEntryStore {
    fn find(&self, code: &str) -> Option<TieredRetentionEntry> {
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

    fn save(&self, item: &TieredRetentionEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredRetentionEntry> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_retention_entry_store(
    inner: Rc<dyn TieredRetentionEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredRetentionEntryStore> {
    Rc::new(CachedTieredRetentionEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredRetentionEntry>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered retention entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionEntryQuery {
    pub label: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredRetentionEntryQuery {
    pub fn matches(&self, item: &TieredRetentionEntry) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredRetentionEntry]) -> Vec<TieredRetentionEntry> {
        let matched = collections::filter(items, |candidate: &TieredRetentionEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_retention_entries(
    store: Rc<dyn TieredRetentionEntryStore>,
    query: &TieredRetentionEntryQuery,
) -> Vec<TieredRetentionEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredRetentionEntryStore> = new_memory_tiered_retention_entry_store();
        let item = new_tiered_retention_entry("code-844", "label-962");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
