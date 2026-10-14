use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary rollback rollups by code.
pub trait PrimaryRollbackRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryRollbackRollup>;
    fn save(&self, item: &PrimaryRollbackRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryRollbackRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryRollbackRollupStore {
    pub items: RefCell<BTreeMap<String, PrimaryRollbackRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryRollbackRollupStore for MemoryPrimaryRollbackRollupStore {
    fn find(&self, code: &str) -> Option<PrimaryRollbackRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryRollbackRollup) {
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

    fn list(&self) -> Vec<PrimaryRollbackRollup> {
        let mut out: Vec<PrimaryRollbackRollup> = Vec::new();
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

pub fn new_memory_primary_rollback_rollup_store() -> Rc<dyn PrimaryRollbackRollupStore> {
    Rc::new(MemoryPrimaryRollbackRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryRollbackRollupStore {
    pub inner: Rc<dyn PrimaryRollbackRollupStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryRollbackRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryRollbackRollupStore for CachedPrimaryRollbackRollupStore {
    fn find(&self, code: &str) -> Option<PrimaryRollbackRollup> {
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

    fn save(&self, item: &PrimaryRollbackRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryRollbackRollup> {
        self.inner.list()
    }
}

pub fn new_cached_primary_rollback_rollup_store(
    inner: Rc<dyn PrimaryRollbackRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryRollbackRollupStore> {
    Rc::new(CachedPrimaryRollbackRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryRollbackRollup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary rollback rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRollbackRollupQuery {
    pub category: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryRollbackRollupQuery {
    pub fn matches(&self, item: &PrimaryRollbackRollup) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryRollbackRollup]) -> Vec<PrimaryRollbackRollup> {
        let matched = collections::filter(items, |candidate: &PrimaryRollbackRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_rollback_rollups(
    store: Rc<dyn PrimaryRollbackRollupStore>,
    query: &PrimaryRollbackRollupQuery,
) -> Vec<PrimaryRollbackRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryRollbackRollupStore> = new_memory_primary_rollback_rollup_store();
        let item = new_primary_rollback_rollup("code-681", "label-963");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
