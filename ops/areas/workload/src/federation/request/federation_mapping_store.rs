use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps federation mappings by code.
pub trait FederationMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FederationMapping>;
    fn save(&self, item: &FederationMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FederationMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFederationMappingStore {
    pub items: RefCell<BTreeMap<String, FederationMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl FederationMappingStore for MemoryFederationMappingStore {
    fn find(&self, code: &str) -> Option<FederationMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FederationMapping) {
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

    fn list(&self) -> Vec<FederationMapping> {
        let mut out: Vec<FederationMapping> = Vec::new();
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

pub fn new_memory_federation_mapping_store() -> Rc<dyn FederationMappingStore> {
    Rc::new(MemoryFederationMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFederationMappingStore {
    pub inner: Rc<dyn FederationMappingStore>,
    pub cache: RefCell<cache::TtlCache<FederationMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FederationMappingStore for CachedFederationMappingStore {
    fn find(&self, code: &str) -> Option<FederationMapping> {
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

    fn save(&self, item: &FederationMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FederationMapping> {
        self.inner.list()
    }
}

pub fn new_cached_federation_mapping_store(
    inner: Rc<dyn FederationMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FederationMappingStore> {
    Rc::new(CachedFederationMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FederationMapping>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects federation mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationMappingQuery {
    pub category: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FederationMappingQuery {
    pub fn matches(&self, item: &FederationMapping) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FederationMapping]) -> Vec<FederationMapping> {
        let matched = collections::filter(items, |candidate: &FederationMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_federation_mappings(
    store: Rc<dyn FederationMappingStore>,
    query: &FederationMappingQuery,
) -> Vec<FederationMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FederationMappingStore> = new_memory_federation_mapping_store();
        let item = new_federation_mapping("code-674", "label-389");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
