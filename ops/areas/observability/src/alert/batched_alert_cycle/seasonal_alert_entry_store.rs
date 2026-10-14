use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal alert entries by code.
pub trait SeasonalAlertEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalAlertEntry>;
    fn save(&self, item: &SeasonalAlertEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalAlertEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalAlertEntryStore {
    pub items: RefCell<BTreeMap<String, SeasonalAlertEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalAlertEntryStore for MemorySeasonalAlertEntryStore {
    fn find(&self, code: &str) -> Option<SeasonalAlertEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalAlertEntry) {
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

    fn list(&self) -> Vec<SeasonalAlertEntry> {
        let mut out: Vec<SeasonalAlertEntry> = Vec::new();
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

pub fn new_memory_seasonal_alert_entry_store() -> Rc<dyn SeasonalAlertEntryStore> {
    Rc::new(MemorySeasonalAlertEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalAlertEntryStore {
    pub inner: Rc<dyn SeasonalAlertEntryStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalAlertEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalAlertEntryStore for CachedSeasonalAlertEntryStore {
    fn find(&self, code: &str) -> Option<SeasonalAlertEntry> {
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

    fn save(&self, item: &SeasonalAlertEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalAlertEntry> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_alert_entry_store(
    inner: Rc<dyn SeasonalAlertEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalAlertEntryStore> {
    Rc::new(CachedSeasonalAlertEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalAlertEntry>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal alert entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertEntryQuery {
    pub cluster: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalAlertEntryQuery {
    pub fn matches(&self, item: &SeasonalAlertEntry) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalAlertEntry]) -> Vec<SeasonalAlertEntry> {
        let matched = collections::filter(items, |candidate: &SeasonalAlertEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_alert_entries(
    store: Rc<dyn SeasonalAlertEntryStore>,
    query: &SeasonalAlertEntryQuery,
) -> Vec<SeasonalAlertEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalAlertEntryStore> = new_memory_seasonal_alert_entry_store();
        let item = new_seasonal_alert_entry("code-790", "name-581");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
