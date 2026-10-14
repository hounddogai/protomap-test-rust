use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default tenant mappings by code.
pub trait DefaultTenantMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultTenantMapping>;
    fn save(&self, item: &DefaultTenantMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultTenantMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultTenantMappingStore {
    pub items: RefCell<BTreeMap<String, DefaultTenantMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultTenantMappingStore for MemoryDefaultTenantMappingStore {
    fn find(&self, code: &str) -> Option<DefaultTenantMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultTenantMapping) {
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

    fn list(&self) -> Vec<DefaultTenantMapping> {
        let mut out: Vec<DefaultTenantMapping> = Vec::new();
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

pub fn new_memory_default_tenant_mapping_store() -> Rc<dyn DefaultTenantMappingStore> {
    Rc::new(MemoryDefaultTenantMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultTenantMappingStore {
    pub inner: Rc<dyn DefaultTenantMappingStore>,
    pub cache: RefCell<cache::TtlCache<DefaultTenantMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultTenantMappingStore for CachedDefaultTenantMappingStore {
    fn find(&self, code: &str) -> Option<DefaultTenantMapping> {
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

    fn save(&self, item: &DefaultTenantMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultTenantMapping> {
        self.inner.list()
    }
}

pub fn new_cached_default_tenant_mapping_store(
    inner: Rc<dyn DefaultTenantMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultTenantMappingStore> {
    Rc::new(CachedDefaultTenantMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultTenantMapping>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default tenant mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantMappingQuery {
    pub cluster: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultTenantMappingQuery {
    pub fn matches(&self, item: &DefaultTenantMapping) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultTenantMapping]) -> Vec<DefaultTenantMapping> {
        let matched = collections::filter(items, |candidate: &DefaultTenantMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_tenant_mappings(
    store: Rc<dyn DefaultTenantMappingStore>,
    query: &DefaultTenantMappingQuery,
) -> Vec<DefaultTenantMapping> {
    query.apply(&store.list())
}
