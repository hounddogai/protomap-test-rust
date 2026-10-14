use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps migration adjustments by code.
pub trait MigrationAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MigrationAdjustment>;
    fn save(&self, item: &MigrationAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MigrationAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMigrationAdjustmentStore {
    pub items: RefCell<BTreeMap<String, MigrationAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl MigrationAdjustmentStore for MemoryMigrationAdjustmentStore {
    fn find(&self, code: &str) -> Option<MigrationAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MigrationAdjustment) {
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

    fn list(&self) -> Vec<MigrationAdjustment> {
        let mut out: Vec<MigrationAdjustment> = Vec::new();
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

pub fn new_memory_migration_adjustment_store() -> Rc<dyn MigrationAdjustmentStore> {
    Rc::new(MemoryMigrationAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMigrationAdjustmentStore {
    pub inner: Rc<dyn MigrationAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<MigrationAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MigrationAdjustmentStore for CachedMigrationAdjustmentStore {
    fn find(&self, code: &str) -> Option<MigrationAdjustment> {
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

    fn save(&self, item: &MigrationAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MigrationAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_migration_adjustment_store(
    inner: Rc<dyn MigrationAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MigrationAdjustmentStore> {
    Rc::new(CachedMigrationAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MigrationAdjustment>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects migration adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAdjustmentQuery {
    pub source: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MigrationAdjustmentQuery {
    pub fn matches(&self, item: &MigrationAdjustment) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[MigrationAdjustment]) -> Vec<MigrationAdjustment> {
        let matched = collections::filter(items, |candidate: &MigrationAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_migration_adjustments(
    store: Rc<dyn MigrationAdjustmentStore>,
    query: &MigrationAdjustmentQuery,
) -> Vec<MigrationAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MigrationAdjustmentStore> = new_memory_migration_adjustment_store();
        let item = new_migration_adjustment("code-112", "title-953");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
