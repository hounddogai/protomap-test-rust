use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps verifier mappings by code.
pub trait VerifierMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<VerifierMapping>;
    fn save(&self, item: &VerifierMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<VerifierMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryVerifierMappingStore {
    pub items: RefCell<BTreeMap<String, VerifierMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl VerifierMappingStore for MemoryVerifierMappingStore {
    fn find(&self, code: &str) -> Option<VerifierMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &VerifierMapping) {
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

    fn list(&self) -> Vec<VerifierMapping> {
        let mut out: Vec<VerifierMapping> = Vec::new();
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

pub fn new_memory_verifier_mapping_store() -> Rc<dyn VerifierMappingStore> {
    Rc::new(MemoryVerifierMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedVerifierMappingStore {
    pub inner: Rc<dyn VerifierMappingStore>,
    pub cache: RefCell<cache::TtlCache<VerifierMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl VerifierMappingStore for CachedVerifierMappingStore {
    fn find(&self, code: &str) -> Option<VerifierMapping> {
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

    fn save(&self, item: &VerifierMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<VerifierMapping> {
        self.inner.list()
    }
}

pub fn new_cached_verifier_mapping_store(
    inner: Rc<dyn VerifierMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn VerifierMappingStore> {
    Rc::new(CachedVerifierMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<VerifierMapping>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects verifier mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMappingQuery {
    pub locale: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl VerifierMappingQuery {
    pub fn matches(&self, item: &VerifierMapping) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[VerifierMapping]) -> Vec<VerifierMapping> {
        let matched = collections::filter(items, |candidate: &VerifierMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_verifier_mappings(
    store: Rc<dyn VerifierMappingStore>,
    query: &VerifierMappingQuery,
) -> Vec<VerifierMapping> {
    query.apply(&store.list())
}
