use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps canary mappings by code.
pub trait CanaryMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CanaryMapping>;
    fn save(&self, item: &CanaryMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CanaryMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCanaryMappingStore {
    pub items: RefCell<BTreeMap<String, CanaryMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl CanaryMappingStore for MemoryCanaryMappingStore {
    fn find(&self, code: &str) -> Option<CanaryMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CanaryMapping) {
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

    fn list(&self) -> Vec<CanaryMapping> {
        let mut out: Vec<CanaryMapping> = Vec::new();
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

pub fn new_memory_canary_mapping_store() -> Rc<dyn CanaryMappingStore> {
    Rc::new(MemoryCanaryMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCanaryMappingStore {
    pub inner: Rc<dyn CanaryMappingStore>,
    pub cache: RefCell<cache::TtlCache<CanaryMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CanaryMappingStore for CachedCanaryMappingStore {
    fn find(&self, code: &str) -> Option<CanaryMapping> {
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

    fn save(&self, item: &CanaryMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CanaryMapping> {
        self.inner.list()
    }
}

pub fn new_cached_canary_mapping_store(
    inner: Rc<dyn CanaryMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CanaryMappingStore> {
    Rc::new(CachedCanaryMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CanaryMapping>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects canary mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMappingQuery {
    pub name: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CanaryMappingQuery {
    pub fn matches(&self, item: &CanaryMapping) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CanaryMapping]) -> Vec<CanaryMapping> {
        let matched = collections::filter(items, |candidate: &CanaryMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_canary_mappings(store: Rc<dyn CanaryMappingStore>, query: &CanaryMappingQuery) -> Vec<CanaryMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CanaryMappingStore> = new_memory_canary_mapping_store();
        let item = new_canary_mapping("code-401", "name-492");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
