use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps migration batches by code.
pub trait MigrationBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MigrationBatch>;
    fn save(&self, item: &MigrationBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MigrationBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMigrationBatchStore {
    pub items: RefCell<BTreeMap<String, MigrationBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl MigrationBatchStore for MemoryMigrationBatchStore {
    fn find(&self, code: &str) -> Option<MigrationBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MigrationBatch) {
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

    fn list(&self) -> Vec<MigrationBatch> {
        let mut out: Vec<MigrationBatch> = Vec::new();
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

pub fn new_memory_migration_batch_store() -> Rc<dyn MigrationBatchStore> {
    Rc::new(MemoryMigrationBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMigrationBatchStore {
    pub inner: Rc<dyn MigrationBatchStore>,
    pub cache: RefCell<cache::TtlCache<MigrationBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MigrationBatchStore for CachedMigrationBatchStore {
    fn find(&self, code: &str) -> Option<MigrationBatch> {
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

    fn save(&self, item: &MigrationBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MigrationBatch> {
        self.inner.list()
    }
}

pub fn new_cached_migration_batch_store(
    inner: Rc<dyn MigrationBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MigrationBatchStore> {
    Rc::new(CachedMigrationBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MigrationBatch>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects migration batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBatchQuery {
    pub name: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MigrationBatchQuery {
    pub fn matches(&self, item: &MigrationBatch) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MigrationBatch]) -> Vec<MigrationBatch> {
        let matched = collections::filter(items, |candidate: &MigrationBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_migration_batches(store: Rc<dyn MigrationBatchStore>, query: &MigrationBatchQuery) -> Vec<MigrationBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MigrationBatchStore> = new_memory_migration_batch_store();
        let item = new_migration_batch("code-397", "name-985");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
