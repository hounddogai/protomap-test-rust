use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps token mappings by code.
pub trait TokenMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TokenMapping>;
    fn save(&self, item: &TokenMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TokenMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTokenMappingStore {
    pub items: RefCell<BTreeMap<String, TokenMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl TokenMappingStore for MemoryTokenMappingStore {
    fn find(&self, code: &str) -> Option<TokenMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TokenMapping) {
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

    fn list(&self) -> Vec<TokenMapping> {
        let mut out: Vec<TokenMapping> = Vec::new();
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

pub fn new_memory_token_mapping_store() -> Rc<dyn TokenMappingStore> {
    Rc::new(MemoryTokenMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTokenMappingStore {
    pub inner: Rc<dyn TokenMappingStore>,
    pub cache: RefCell<cache::TtlCache<TokenMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TokenMappingStore for CachedTokenMappingStore {
    fn find(&self, code: &str) -> Option<TokenMapping> {
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

    fn save(&self, item: &TokenMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TokenMapping> {
        self.inner.list()
    }
}

pub fn new_cached_token_mapping_store(
    inner: Rc<dyn TokenMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TokenMappingStore> {
    Rc::new(CachedTokenMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TokenMapping>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects token mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMappingQuery {
    pub scope: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TokenMappingQuery {
    pub fn matches(&self, item: &TokenMapping) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TokenMapping]) -> Vec<TokenMapping> {
        let matched = collections::filter(items, |candidate: &TokenMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_token_mappings(store: Rc<dyn TokenMappingStore>, query: &TokenMappingQuery) -> Vec<TokenMapping> {
    query.apply(&store.list())
}
