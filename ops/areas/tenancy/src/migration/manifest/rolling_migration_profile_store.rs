use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling migration profiles by code.
pub trait RollingMigrationProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingMigrationProfile>;
    fn save(&self, item: &RollingMigrationProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingMigrationProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingMigrationProfileStore {
    pub items: RefCell<BTreeMap<String, RollingMigrationProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingMigrationProfileStore for MemoryRollingMigrationProfileStore {
    fn find(&self, code: &str) -> Option<RollingMigrationProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingMigrationProfile) {
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

    fn list(&self) -> Vec<RollingMigrationProfile> {
        let mut out: Vec<RollingMigrationProfile> = Vec::new();
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

pub fn new_memory_rolling_migration_profile_store() -> Rc<dyn RollingMigrationProfileStore> {
    Rc::new(MemoryRollingMigrationProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingMigrationProfileStore {
    pub inner: Rc<dyn RollingMigrationProfileStore>,
    pub cache: RefCell<cache::TtlCache<RollingMigrationProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingMigrationProfileStore for CachedRollingMigrationProfileStore {
    fn find(&self, code: &str) -> Option<RollingMigrationProfile> {
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

    fn save(&self, item: &RollingMigrationProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingMigrationProfile> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_migration_profile_store(
    inner: Rc<dyn RollingMigrationProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingMigrationProfileStore> {
    Rc::new(CachedRollingMigrationProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingMigrationProfile>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling migration profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationProfileQuery {
    pub source: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingMigrationProfileQuery {
    pub fn matches(&self, item: &RollingMigrationProfile) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[RollingMigrationProfile]) -> Vec<RollingMigrationProfile> {
        let matched = collections::filter(items, |candidate: &RollingMigrationProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_migration_profiles(
    store: Rc<dyn RollingMigrationProfileStore>,
    query: &RollingMigrationProfileQuery,
) -> Vec<RollingMigrationProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingMigrationProfileStore> = new_memory_rolling_migration_profile_store();
        let item = new_rolling_migration_profile("code-210", "name-199");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
