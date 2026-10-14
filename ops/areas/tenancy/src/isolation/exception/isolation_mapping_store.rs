use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps isolation mappings by code.
pub trait IsolationMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IsolationMapping>;
    fn save(&self, item: &IsolationMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IsolationMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIsolationMappingStore {
    pub items: RefCell<BTreeMap<String, IsolationMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl IsolationMappingStore for MemoryIsolationMappingStore {
    fn find(&self, code: &str) -> Option<IsolationMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IsolationMapping) {
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

    fn list(&self) -> Vec<IsolationMapping> {
        let mut out: Vec<IsolationMapping> = Vec::new();
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

pub fn new_memory_isolation_mapping_store() -> Rc<dyn IsolationMappingStore> {
    Rc::new(MemoryIsolationMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIsolationMappingStore {
    pub inner: Rc<dyn IsolationMappingStore>,
    pub cache: RefCell<cache::TtlCache<IsolationMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IsolationMappingStore for CachedIsolationMappingStore {
    fn find(&self, code: &str) -> Option<IsolationMapping> {
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

    fn save(&self, item: &IsolationMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IsolationMapping> {
        self.inner.list()
    }
}

pub fn new_cached_isolation_mapping_store(
    inner: Rc<dyn IsolationMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IsolationMappingStore> {
    Rc::new(CachedIsolationMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IsolationMapping>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects isolation mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMappingQuery {
    pub label: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IsolationMappingQuery {
    pub fn matches(&self, item: &IsolationMapping) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IsolationMapping]) -> Vec<IsolationMapping> {
        let matched = collections::filter(items, |candidate: &IsolationMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_isolation_mappings(
    store: Rc<dyn IsolationMappingStore>,
    query: &IsolationMappingQuery,
) -> Vec<IsolationMapping> {
    query.apply(&store.list())
}
