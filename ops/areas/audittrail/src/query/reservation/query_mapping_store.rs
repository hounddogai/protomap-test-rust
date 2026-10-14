use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query mappings by code.
pub trait QueryMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryMapping>;
    fn save(&self, item: &QueryMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryMappingStore {
    pub items: RefCell<BTreeMap<String, QueryMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryMappingStore for MemoryQueryMappingStore {
    fn find(&self, code: &str) -> Option<QueryMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryMapping) {
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

    fn list(&self) -> Vec<QueryMapping> {
        let mut out: Vec<QueryMapping> = Vec::new();
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

pub fn new_memory_query_mapping_store() -> Rc<dyn QueryMappingStore> {
    Rc::new(MemoryQueryMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryMappingStore {
    pub inner: Rc<dyn QueryMappingStore>,
    pub cache: RefCell<cache::TtlCache<QueryMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryMappingStore for CachedQueryMappingStore {
    fn find(&self, code: &str) -> Option<QueryMapping> {
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

    fn save(&self, item: &QueryMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryMapping> {
        self.inner.list()
    }
}

pub fn new_cached_query_mapping_store(
    inner: Rc<dyn QueryMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryMappingStore> {
    Rc::new(CachedQueryMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryMapping>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryMappingQuery {
    pub source: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryMappingQuery {
    pub fn matches(&self, item: &QueryMapping) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QueryMapping]) -> Vec<QueryMapping> {
        let matched = collections::filter(items, |candidate: &QueryMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_mappings(store: Rc<dyn QueryMappingStore>, query: &QueryMappingQuery) -> Vec<QueryMapping> {
    query.apply(&store.list())
}
