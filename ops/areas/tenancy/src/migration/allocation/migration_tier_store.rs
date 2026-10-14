use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps migration tiers by code.
pub trait MigrationTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MigrationTier>;
    fn save(&self, item: &MigrationTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MigrationTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMigrationTierStore {
    pub items: RefCell<BTreeMap<String, MigrationTier>>,
    pub order: RefCell<Vec<String>>,
}

impl MigrationTierStore for MemoryMigrationTierStore {
    fn find(&self, code: &str) -> Option<MigrationTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MigrationTier) {
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

    fn list(&self) -> Vec<MigrationTier> {
        let mut out: Vec<MigrationTier> = Vec::new();
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

pub fn new_memory_migration_tier_store() -> Rc<dyn MigrationTierStore> {
    Rc::new(MemoryMigrationTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMigrationTierStore {
    pub inner: Rc<dyn MigrationTierStore>,
    pub cache: RefCell<cache::TtlCache<MigrationTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MigrationTierStore for CachedMigrationTierStore {
    fn find(&self, code: &str) -> Option<MigrationTier> {
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

    fn save(&self, item: &MigrationTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MigrationTier> {
        self.inner.list()
    }
}

pub fn new_cached_migration_tier_store(
    inner: Rc<dyn MigrationTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MigrationTierStore> {
    Rc::new(CachedMigrationTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MigrationTier>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects migration tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTierQuery {
    pub name: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MigrationTierQuery {
    pub fn matches(&self, item: &MigrationTier) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MigrationTier]) -> Vec<MigrationTier> {
        let matched = collections::filter(items, |candidate: &MigrationTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_migration_tiers(store: Rc<dyn MigrationTierStore>, query: &MigrationTierQuery) -> Vec<MigrationTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MigrationTierStore> = new_memory_migration_tier_store();
        let item = new_migration_tier("code-273", "name-455");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
