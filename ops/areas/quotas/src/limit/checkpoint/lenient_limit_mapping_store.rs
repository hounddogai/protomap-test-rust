use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient limit mappings by code.
pub trait LenientLimitMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientLimitMapping>;
    fn save(&self, item: &LenientLimitMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientLimitMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientLimitMappingStore {
    pub items: RefCell<BTreeMap<String, LenientLimitMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientLimitMappingStore for MemoryLenientLimitMappingStore {
    fn find(&self, code: &str) -> Option<LenientLimitMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientLimitMapping) {
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

    fn list(&self) -> Vec<LenientLimitMapping> {
        let mut out: Vec<LenientLimitMapping> = Vec::new();
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

pub fn new_memory_lenient_limit_mapping_store() -> Rc<dyn LenientLimitMappingStore> {
    Rc::new(MemoryLenientLimitMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientLimitMappingStore {
    pub inner: Rc<dyn LenientLimitMappingStore>,
    pub cache: RefCell<cache::TtlCache<LenientLimitMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientLimitMappingStore for CachedLenientLimitMappingStore {
    fn find(&self, code: &str) -> Option<LenientLimitMapping> {
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

    fn save(&self, item: &LenientLimitMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientLimitMapping> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_limit_mapping_store(
    inner: Rc<dyn LenientLimitMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientLimitMappingStore> {
    Rc::new(CachedLenientLimitMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientLimitMapping>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient limit mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitMappingQuery {
    pub region: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientLimitMappingQuery {
    pub fn matches(&self, item: &LenientLimitMapping) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[LenientLimitMapping]) -> Vec<LenientLimitMapping> {
        let matched = collections::filter(items, |candidate: &LenientLimitMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_limit_mappings(
    store: Rc<dyn LenientLimitMappingStore>,
    query: &LenientLimitMappingQuery,
) -> Vec<LenientLimitMapping> {
    query.apply(&store.list())
}
