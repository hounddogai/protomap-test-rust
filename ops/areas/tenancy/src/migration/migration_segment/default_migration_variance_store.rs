use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default migration variances by code.
pub trait DefaultMigrationVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultMigrationVariance>;
    fn save(&self, item: &DefaultMigrationVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultMigrationVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultMigrationVarianceStore {
    pub items: RefCell<BTreeMap<String, DefaultMigrationVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultMigrationVarianceStore for MemoryDefaultMigrationVarianceStore {
    fn find(&self, code: &str) -> Option<DefaultMigrationVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultMigrationVariance) {
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

    fn list(&self) -> Vec<DefaultMigrationVariance> {
        let mut out: Vec<DefaultMigrationVariance> = Vec::new();
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

pub fn new_memory_default_migration_variance_store() -> Rc<dyn DefaultMigrationVarianceStore> {
    Rc::new(MemoryDefaultMigrationVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultMigrationVarianceStore {
    pub inner: Rc<dyn DefaultMigrationVarianceStore>,
    pub cache: RefCell<cache::TtlCache<DefaultMigrationVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultMigrationVarianceStore for CachedDefaultMigrationVarianceStore {
    fn find(&self, code: &str) -> Option<DefaultMigrationVariance> {
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

    fn save(&self, item: &DefaultMigrationVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultMigrationVariance> {
        self.inner.list()
    }
}

pub fn new_cached_default_migration_variance_store(
    inner: Rc<dyn DefaultMigrationVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultMigrationVarianceStore> {
    Rc::new(CachedDefaultMigrationVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultMigrationVariance>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default migration variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationVarianceQuery {
    pub segment: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultMigrationVarianceQuery {
    pub fn matches(&self, item: &DefaultMigrationVariance) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultMigrationVariance]) -> Vec<DefaultMigrationVariance> {
        let matched = collections::filter(items, |candidate: &DefaultMigrationVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_migration_variances(
    store: Rc<dyn DefaultMigrationVarianceStore>,
    query: &DefaultMigrationVarianceQuery,
) -> Vec<DefaultMigrationVariance> {
    query.apply(&store.list())
}
