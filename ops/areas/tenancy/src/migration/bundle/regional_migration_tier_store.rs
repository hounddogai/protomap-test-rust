use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional migration tiers by code.
pub trait RegionalMigrationTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalMigrationTier>;
    fn save(&self, item: &RegionalMigrationTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalMigrationTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalMigrationTierStore {
    pub items: RefCell<BTreeMap<String, RegionalMigrationTier>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalMigrationTierStore for MemoryRegionalMigrationTierStore {
    fn find(&self, code: &str) -> Option<RegionalMigrationTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalMigrationTier) {
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

    fn list(&self) -> Vec<RegionalMigrationTier> {
        let mut out: Vec<RegionalMigrationTier> = Vec::new();
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

pub fn new_memory_regional_migration_tier_store() -> Rc<dyn RegionalMigrationTierStore> {
    Rc::new(MemoryRegionalMigrationTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalMigrationTierStore {
    pub inner: Rc<dyn RegionalMigrationTierStore>,
    pub cache: RefCell<cache::TtlCache<RegionalMigrationTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalMigrationTierStore for CachedRegionalMigrationTierStore {
    fn find(&self, code: &str) -> Option<RegionalMigrationTier> {
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

    fn save(&self, item: &RegionalMigrationTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalMigrationTier> {
        self.inner.list()
    }
}

pub fn new_cached_regional_migration_tier_store(
    inner: Rc<dyn RegionalMigrationTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalMigrationTierStore> {
    Rc::new(CachedRegionalMigrationTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalMigrationTier>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional migration tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationTierQuery {
    pub namespace: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalMigrationTierQuery {
    pub fn matches(&self, item: &RegionalMigrationTier) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalMigrationTier]) -> Vec<RegionalMigrationTier> {
        let matched = collections::filter(items, |candidate: &RegionalMigrationTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_migration_tiers(
    store: Rc<dyn RegionalMigrationTierStore>,
    query: &RegionalMigrationTierQuery,
) -> Vec<RegionalMigrationTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalMigrationTierStore> = new_memory_regional_migration_tier_store();
        let item = new_regional_migration_tier("code-412", "title-823");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
