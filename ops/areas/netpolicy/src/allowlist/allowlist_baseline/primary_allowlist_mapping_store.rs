use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary allowlist mappings by code.
pub trait PrimaryAllowlistMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryAllowlistMapping>;
    fn save(&self, item: &PrimaryAllowlistMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryAllowlistMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryAllowlistMappingStore {
    pub items: RefCell<BTreeMap<String, PrimaryAllowlistMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryAllowlistMappingStore for MemoryPrimaryAllowlistMappingStore {
    fn find(&self, code: &str) -> Option<PrimaryAllowlistMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryAllowlistMapping) {
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

    fn list(&self) -> Vec<PrimaryAllowlistMapping> {
        let mut out: Vec<PrimaryAllowlistMapping> = Vec::new();
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

pub fn new_memory_primary_allowlist_mapping_store() -> Rc<dyn PrimaryAllowlistMappingStore> {
    Rc::new(MemoryPrimaryAllowlistMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryAllowlistMappingStore {
    pub inner: Rc<dyn PrimaryAllowlistMappingStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryAllowlistMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryAllowlistMappingStore for CachedPrimaryAllowlistMappingStore {
    fn find(&self, code: &str) -> Option<PrimaryAllowlistMapping> {
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

    fn save(&self, item: &PrimaryAllowlistMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryAllowlistMapping> {
        self.inner.list()
    }
}

pub fn new_cached_primary_allowlist_mapping_store(
    inner: Rc<dyn PrimaryAllowlistMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryAllowlistMappingStore> {
    Rc::new(CachedPrimaryAllowlistMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryAllowlistMapping>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary allowlist mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAllowlistMappingQuery {
    pub title: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryAllowlistMappingQuery {
    pub fn matches(&self, item: &PrimaryAllowlistMapping) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryAllowlistMapping]) -> Vec<PrimaryAllowlistMapping> {
        let matched = collections::filter(items, |candidate: &PrimaryAllowlistMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_allowlist_mappings(
    store: Rc<dyn PrimaryAllowlistMappingStore>,
    query: &PrimaryAllowlistMappingQuery,
) -> Vec<PrimaryAllowlistMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryAllowlistMappingStore> = new_memory_primary_allowlist_mapping_store();
        let item = new_primary_allowlist_mapping("code-625", "title-636");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
