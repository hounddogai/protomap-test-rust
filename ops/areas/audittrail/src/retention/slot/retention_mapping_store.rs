use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention mappings by code.
pub trait RetentionMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionMapping>;
    fn save(&self, item: &RetentionMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionMappingStore {
    pub items: RefCell<BTreeMap<String, RetentionMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionMappingStore for MemoryRetentionMappingStore {
    fn find(&self, code: &str) -> Option<RetentionMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionMapping) {
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

    fn list(&self) -> Vec<RetentionMapping> {
        let mut out: Vec<RetentionMapping> = Vec::new();
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

pub fn new_memory_retention_mapping_store() -> Rc<dyn RetentionMappingStore> {
    Rc::new(MemoryRetentionMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionMappingStore {
    pub inner: Rc<dyn RetentionMappingStore>,
    pub cache: RefCell<cache::TtlCache<RetentionMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionMappingStore for CachedRetentionMappingStore {
    fn find(&self, code: &str) -> Option<RetentionMapping> {
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

    fn save(&self, item: &RetentionMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionMapping> {
        self.inner.list()
    }
}

pub fn new_cached_retention_mapping_store(
    inner: Rc<dyn RetentionMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionMappingStore> {
    Rc::new(CachedRetentionMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionMapping>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMappingQuery {
    pub region: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionMappingQuery {
    pub fn matches(&self, item: &RetentionMapping) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[RetentionMapping]) -> Vec<RetentionMapping> {
        let matched = collections::filter(items, |candidate: &RetentionMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_mappings(
    store: Rc<dyn RetentionMappingStore>,
    query: &RetentionMappingQuery,
) -> Vec<RetentionMapping> {
    query.apply(&store.list())
}
