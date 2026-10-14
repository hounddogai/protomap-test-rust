use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps migration profiles by code.
pub trait MigrationProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MigrationProfile>;
    fn save(&self, item: &MigrationProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MigrationProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMigrationProfileStore {
    pub items: RefCell<BTreeMap<String, MigrationProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl MigrationProfileStore for MemoryMigrationProfileStore {
    fn find(&self, code: &str) -> Option<MigrationProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MigrationProfile) {
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

    fn list(&self) -> Vec<MigrationProfile> {
        let mut out: Vec<MigrationProfile> = Vec::new();
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

pub fn new_memory_migration_profile_store() -> Rc<dyn MigrationProfileStore> {
    Rc::new(MemoryMigrationProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMigrationProfileStore {
    pub inner: Rc<dyn MigrationProfileStore>,
    pub cache: RefCell<cache::TtlCache<MigrationProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MigrationProfileStore for CachedMigrationProfileStore {
    fn find(&self, code: &str) -> Option<MigrationProfile> {
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

    fn save(&self, item: &MigrationProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MigrationProfile> {
        self.inner.list()
    }
}

pub fn new_cached_migration_profile_store(
    inner: Rc<dyn MigrationProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MigrationProfileStore> {
    Rc::new(CachedMigrationProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MigrationProfile>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects migration profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProfileQuery {
    pub label: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MigrationProfileQuery {
    pub fn matches(&self, item: &MigrationProfile) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MigrationProfile]) -> Vec<MigrationProfile> {
        let matched = collections::filter(items, |candidate: &MigrationProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_migration_profiles(
    store: Rc<dyn MigrationProfileStore>,
    query: &MigrationProfileQuery,
) -> Vec<MigrationProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MigrationProfileStore> = new_memory_migration_profile_store();
        let item = new_migration_profile("code-250", "label-387");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
