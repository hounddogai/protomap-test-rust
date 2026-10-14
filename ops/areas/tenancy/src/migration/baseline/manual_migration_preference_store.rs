use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual migration preferences by code.
pub trait ManualMigrationPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualMigrationPreference>;
    fn save(&self, item: &ManualMigrationPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualMigrationPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualMigrationPreferenceStore {
    pub items: RefCell<BTreeMap<String, ManualMigrationPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualMigrationPreferenceStore for MemoryManualMigrationPreferenceStore {
    fn find(&self, code: &str) -> Option<ManualMigrationPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualMigrationPreference) {
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

    fn list(&self) -> Vec<ManualMigrationPreference> {
        let mut out: Vec<ManualMigrationPreference> = Vec::new();
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

pub fn new_memory_manual_migration_preference_store() -> Rc<dyn ManualMigrationPreferenceStore> {
    Rc::new(MemoryManualMigrationPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualMigrationPreferenceStore {
    pub inner: Rc<dyn ManualMigrationPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<ManualMigrationPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualMigrationPreferenceStore for CachedManualMigrationPreferenceStore {
    fn find(&self, code: &str) -> Option<ManualMigrationPreference> {
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

    fn save(&self, item: &ManualMigrationPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualMigrationPreference> {
        self.inner.list()
    }
}

pub fn new_cached_manual_migration_preference_store(
    inner: Rc<dyn ManualMigrationPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualMigrationPreferenceStore> {
    Rc::new(CachedManualMigrationPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualMigrationPreference>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual migration preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationPreferenceQuery {
    pub title: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualMigrationPreferenceQuery {
    pub fn matches(&self, item: &ManualMigrationPreference) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualMigrationPreference]) -> Vec<ManualMigrationPreference> {
        let matched = collections::filter(items, |candidate: &ManualMigrationPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_migration_preferences(
    store: Rc<dyn ManualMigrationPreferenceStore>,
    query: &ManualMigrationPreferenceQuery,
) -> Vec<ManualMigrationPreference> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualMigrationPreferenceStore> = new_memory_manual_migration_preference_store();
        let item = new_manual_migration_preference("code-681", "title-766");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
