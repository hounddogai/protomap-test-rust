use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps migration groups by code.
pub trait MigrationGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MigrationGroup>;
    fn save(&self, item: &MigrationGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MigrationGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMigrationGroupStore {
    pub items: RefCell<BTreeMap<String, MigrationGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl MigrationGroupStore for MemoryMigrationGroupStore {
    fn find(&self, code: &str) -> Option<MigrationGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MigrationGroup) {
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

    fn list(&self) -> Vec<MigrationGroup> {
        let mut out: Vec<MigrationGroup> = Vec::new();
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

pub fn new_memory_migration_group_store() -> Rc<dyn MigrationGroupStore> {
    Rc::new(MemoryMigrationGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMigrationGroupStore {
    pub inner: Rc<dyn MigrationGroupStore>,
    pub cache: RefCell<cache::TtlCache<MigrationGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MigrationGroupStore for CachedMigrationGroupStore {
    fn find(&self, code: &str) -> Option<MigrationGroup> {
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

    fn save(&self, item: &MigrationGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MigrationGroup> {
        self.inner.list()
    }
}

pub fn new_cached_migration_group_store(
    inner: Rc<dyn MigrationGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MigrationGroupStore> {
    Rc::new(CachedMigrationGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MigrationGroup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects migration groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationGroupQuery {
    pub title: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MigrationGroupQuery {
    pub fn matches(&self, item: &MigrationGroup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MigrationGroup]) -> Vec<MigrationGroup> {
        let matched = collections::filter(items, |candidate: &MigrationGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_migration_groups(store: Rc<dyn MigrationGroupStore>, query: &MigrationGroupQuery) -> Vec<MigrationGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MigrationGroupStore> = new_memory_migration_group_store();
        let item = new_migration_group("code-475", "title-476");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
